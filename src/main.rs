use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};
use ahash::AHashMap;
use smallvec::SmallVec;
use rayon::prelude::*;
use std::time::Instant;
use std::hash::{Hash, Hasher};
use twox_hash::XxHash64;
use half::f16;

mod dataset;
mod tests;
mod storage;
mod embedding;
mod inference_engine;
mod gliner_ner;
use storage::{generate_binary_dataset, StorageEngine, ChaosFingerprint};
use dataset::{get_tech_domain_data, get_social_domain_data, get_history_domain_data, get_value_domain_data, get_daily_domain_data, get_timeline_domain_data, get_ontology_data};
use embedding::CandleModel;
use gliner_ner::GlinerEngine;

// ============================================================================
// 1. 语义指纹 (SimHash V2: Partitioned Multimodal)
// ============================================================================

pub struct SimHash;

impl SimHash {
    pub const MASK_SEMANTIC: u64 = 0xFFFFFFFF;
    pub const MASK_TEMPORAL: u64 = 0xFFFF00000000; // [32-47]: 时间区 (仅时间 - V3 中已移除位置)
    pub const MASK_AFFECTIVE: u64 = 0x00FF000000000000;
    pub const MASK_TYPE: u64 = 0xFF00000000000000;

    // --- 实体类型常量 ---
    pub const TYPE_UNKNOWN: u8 = 0x00;
    pub const TYPE_PERSON: u8 = 0x01;    // 人物/身份
    pub const TYPE_TECH: u8 = 0x02;      // 技术/概念
    pub const TYPE_EVENT: u8 = 0x03;     // 事件/动作
    pub const TYPE_LOCATION: u8 = 0x04;  // 地点
    pub const TYPE_OBJECT: u8 = 0x05;    // 物件
    pub const TYPE_VALUES: u8 = 0x06;    // 价值观

    // --- 边类型常量 (V3.5 类型化边 - 简化版) ---
    pub const EDGE_REPRESENTATION: u8 = 0; // 表征 (Representation) - "看到 B 可能会想到 A" (单向/非等价)
    pub const EDGE_EQUALITY: u8 = 1;       // 等价 (Equality) - "A 就是 B" (双向/零损耗)
    pub const EDGE_INHIBITION: u8 = 255;   // 抑制 (Inhibition) - "A 与 B 互斥" (双向/负反馈)

    // --- 情感常量 (Plutchik 情感轮位图 - 已调整) ---
    pub const EMOTION_JOY: u8          = 1 << 0; // 喜悦
    pub const EMOTION_SHY: u8          = 1 << 1; // 害羞
    pub const EMOTION_FEAR: u8         = 1 << 2; // 害怕
    pub const EMOTION_SURPRISE: u8     = 1 << 3; // 惊讶
    pub const EMOTION_SADNESS: u8      = 1 << 4; // 难过
    pub const EMOTION_DISGUST: u8      = 1 << 5; // 讨厌
    pub const EMOTION_ANGER: u8        = 1 << 6; // 生气
    pub const EMOTION_ANTICIPATION: u8 = 1 << 7; // 期待

    /// 计算多模态分区指纹 (64位)
    /// [0-31]: 语义区 (文本)
    /// [32-47]: 时间区 (时间)
    /// [48-55]: 情感区 (情感)
    /// [56-63]: 类型区 (实体类型)
    pub fn compute_multimodal(text: &str, timestamp: u64, emotion_val: u8, type_val: u8) -> u64 {
        let mut fp = 0u64;

        // 1. 语义区 [0-31] (32位)
        let semantic_hash = Self::compute_text_hash_32(text);
        fp |= (semantic_hash as u64) & Self::MASK_SEMANTIC;

        // 2. 时间区 [32-47] (16位) - 仅保留时间
        if timestamp > 0 {
            let t_hash = Self::compute_temporal_hash(timestamp);
            fp |= ((t_hash as u64) << 32) & Self::MASK_TEMPORAL;
        }

        // 3. 情感区 [48-55] (8位)
        fp |= ((emotion_val as u64) << 48) & Self::MASK_AFFECTIVE;

        // 4. 类型区 [56-63] (8位)
        fp |= ((type_val as u64) << 56) & Self::MASK_TYPE;

        fp
    }

    /// 针对查询字符串的智能指纹生成 (增强的时间感知)
    /// ref_time: 外部传入的参考时间戳（现实时间或叙事时间），用于解析相对时间
    pub fn compute_for_query(query: &str, ref_time: u64) -> u64 {
        let mut timestamp = 0u64;
        let mut type_val = Self::TYPE_UNKNOWN;

        let query_lower = query.to_lowercase();

        // --- 1. 相对时间解析 (相对时间分辨率) ---
        // 只有当 ref_time 有效 (>0) 时才启用相对时间解析
        if ref_time > 0 {
            // 0. 今天/今日/此刻 (当前)
            if query_lower.contains("今天") || query_lower.contains("今日") || query_lower.contains("today") || 
               query_lower.contains("now") || query_lower.contains("此刻") || query_lower.contains("当前") {
                timestamp = ref_time;
            }
            // 1. 昨天/昨日 (1天前)
            else if query_lower.contains("昨天") || query_lower.contains("昨日") || query_lower.contains("yesterday") {
                timestamp = ref_time.saturating_sub(86400);
            }
            // 2. 前天/前日 (2天前)
            else if query_lower.contains("前天") || query_lower.contains("前日") {
                timestamp = ref_time.saturating_sub(172800);
            }
            // 3. 大前天 (3天前)
            else if query_lower.contains("大前天") {
                timestamp = ref_time.saturating_sub(259200);
            }
            // 4. 前几天/最近 (约3天前) - 模糊匹配
            else if query_lower.contains("前几天") || query_lower.contains("最近") || query_lower.contains("recently") {
                timestamp = ref_time.saturating_sub(259200);
            }
            // 5. 上周/上星期 (7天前)
            else if query_lower.contains("上周") || query_lower.contains("last week") {
                timestamp = ref_time.saturating_sub(604800);
            }
            // 6. 上个月/上月 (30天前)
            else if query_lower.contains("上个月") || query_lower.contains("上月") || query_lower.contains("last month") {
                timestamp = ref_time.saturating_sub(2592000);
            }
            // 7. 去年 (365天前)
            else if query_lower.contains("去年") || query_lower.contains("last year") {
                timestamp = ref_time.saturating_sub(31536000); 
            }
            // 8. 前年 (2年前)
            else if query_lower.contains("前年") {
                timestamp = ref_time.saturating_sub(63072000); 
            }
            // 9. 刚才/刚刚 (刚才 - 1分钟前)
            else if query_lower.contains("刚才") || query_lower.contains("刚刚") || query_lower.contains("just now") {
                timestamp = ref_time.saturating_sub(60); 
            }
            // 10. 早上/上午 (早晨 - 假设当天的 9:00 AM)
            // 这是一个粗略的锚点，如果 ref_time 已经是当天，我们其实只需要当天的日期部分
            // 但为了简化，这里暂时指向 ref_time (当天)
            else if query_lower.contains("早上") || query_lower.contains("上午") || query_lower.contains("morning") {
                 timestamp = ref_time; 
            }
        }

        // --- 2. 绝对时间解析 (绝对时间回退) ---
        // 只有在相对时间未命中时才尝试绝对年份匹配
        if timestamp == 0 {
            if query_lower.contains("2024") { timestamp = 1704067200; } // 2024-01-01
            if query_lower.contains("2025") { timestamp = 1735689600; } // 2025-01-01
            if query_lower.contains("2026") { timestamp = 1767225600; } // 2026-01-01
        }
        
        // 模拟情感提取 (Plutchik 情感轮)
        let emotion = Self::extract_emotion(&query_lower);

        // 模拟类型推断

        if query_lower.contains("pero") || query_lower.contains("用户") || query_lower.contains("女孩") {
            type_val = Self::TYPE_PERSON;
        } else if query_lower.contains("rust") || query_lower.contains("代码") || query_lower.contains("算法") {
            type_val = Self::TYPE_TECH;
        } else if query_lower.contains("事情") || query_lower.contains("发生") {
            type_val = Self::TYPE_EVENT;
        } else if query_lower.contains("蝴蝶结") || query_lower.contains("键盘") {
            type_val = Self::TYPE_OBJECT;
        }

        Self::compute_multimodal(&query_lower, timestamp, emotion, type_val)
    }

    fn get_emotion_keywords() -> &'static [(u8, &'static [&'static str])] {
        &[
            (Self::EMOTION_JOY, &[
                "开心", "欣慰", "棒", "成功", "快乐", "幸福", "高兴", "喜悦",
                "兴奋", "爽", "nice", "happy", "good", "great", "满意", "舒服", 
                "赞", "完美", "优秀", "庆祝", "哈哈", "lol", "awesome", "perfect", 
                "satisfied", "enjoy", "love", "喜欢", "爱", "满足", "得意", "狂喜",
                "luck", "win", "yeah", "酷", "cool", "fun", "funny", "glad", 
                "pleased", "delight", "爽翻", "乐", "best", "wonderful"
            ]),
            (Self::EMOTION_SHY, &[ // 信任/接纳
                "害羞", "不好意思", "脸红", "谢谢", "感谢", "信任", "依靠",
                "抱歉", "依赖", "相信", "敬佩", "认同", "support", "trust", 
                "believe", "thanks", "agree", "accept", "admire", "忠诚", "老实",
                "可靠", "靠谱", "实在", "真诚", "坦诚", "honest", "loyal", "faith",
                "true", "real", "safe", "secure", "respect", "appreciate"
            ]),
            (Self::EMOTION_FEAR, &[
                "害怕", "担心", "焦虑", "恐惧", "紧张", "慌", "吓",
                "恐怖", "吓人", "没底", "忐忑", "不安", "危机", "风险",
                "afraid", "scared", "worry", "nervous", "panic", "risk",
                "惊慌", "胆怯", "畏惧", "alarm", "dread", "terror", "怕", "悚",
                "提心吊胆", "danger", "threat", "horror", "anxiety"
            ]),
            (Self::EMOTION_SURPRISE, &[
                "没想到", "竟然", "惊讶", "震惊", "卧槽", "牛逼", "哇",
                "居然", "意外", "奇迹", "神奇", "amazing", "wow", "omg", 
                "surprise", "shock", "incredible", "unbelievable", "猝不及防",
                "愣住", "startle", "astonish", "惊呆", "傻眼", "措手不及",
                "wonder", "stun", "sudden", "unexpected"
            ]),
            (Self::EMOTION_SADNESS, &[
                "难过", "低落", "失望", "遗憾", "伤心", "痛苦", "悲伤", "哭",
                "泪", "可惜", "抑郁", "沮丧", "孤独", "惨", "完蛋", "心碎",
                "depressed", "sad", "sorry", "miss", "fail", "lost", "lonely",
                "哀伤", "凄凉", "苦恼", "grief", "mourn", "upset", "痛", "郁闷",
                "心酸", "hurt", "cry", "weep", "pity", "heartbroken"
            ]),
            (Self::EMOTION_DISGUST, &[
                "讨厌", "不喜欢", "烂", "恶心", "烦", "滚", "垃圾", "差劲",
                "无语", "鄙视", "恶劣", "丑陋", "shit", "hate", "dislike", 
                "suck", "bad", "nasty", "awful", "boring", "反感", "鄙夷",
                "唾弃", "revulsion", "loathe", "呸", "滚蛋", "废物", "trash",
                "garbage", "gross", "yuck", "sick"
            ]),
            (Self::EMOTION_ANGER, &[
                "生气", "恼火", "不爽", "愤怒", "怒", "恨", "气死",
                "暴躁", "妈的", "靠", "投诉", "furious", "mad", "angry", 
                "rage", "fuck", "damn", "火大", "发飙", "irritate", "resent",
                "outrage", "气炸", "找死", "闭嘴", "shut up", "piss off", "annoy"
            ]),
            (Self::EMOTION_ANTICIPATION, &[
                "期待", "愿景", "未来", "规划", "希望", "想要", "盼望", "准备",
                "计划", "打算", "目标", "憧憬", "等待", "wait", "plan", "goal", 
                "hope", "expect", "ready", "wish", "渴望", "预感", "祈祷",
                "祝愿", "look forward", "desire", "pray", "dream", "seek"
            ]),
        ]
    }

    /// 从文本中提取情感 (Plutchik's Wheel)
    pub fn extract_emotion(text: &str) -> u8 {
        let mut emotion = 0u8;
        let text_lower = text.to_lowercase();
        
        for &(flag, keywords) in Self::get_emotion_keywords() {
            for &keyword in keywords {
                if text_lower.contains(keyword) {
                    emotion |= flag;
                    break;
                }
            }
        }

        emotion
    }

    /// 传统的 SimHash 计算 (仅用于语义区，压缩到 32 位)
    pub fn compute_text_hash_32(text: &str) -> u32 {
        let text_lower = text.to_lowercase();
        let mut v = [0i32; 32];
        
        for word in text_lower.split_whitespace() {
            Self::update_v_32(&mut v, word);
        }
        // 处理中文等无空格字符
        for c in text_lower.chars() {
            let mut buf = [0u8; 4];
            let s = c.encode_utf8(&mut buf);
            Self::update_v_32(&mut v, s);
        }

        let mut finger_print = 0u32;
        for i in 0..32 {
            if v[i] > 0 {
                finger_print |= 1 << i;
            }
        }
        finger_print
    }

    /// 兼容旧版接口 (仅计算文本，其他默认为 0)
    pub fn compute(text: &str) -> u64 {
        Self::compute_multimodal(text, 0, 0, 0)
    }

    fn update_v_32(v: &mut [i32; 32], token: &str) {
        let mut hasher = XxHash64::with_seed(0);
        token.hash(&mut hasher);
        let hash = hasher.finish();
        
        for i in 0..32 {
            let bit = (hash >> i) & 1;
            if bit == 1 {
                v[i] += 1;
            } else {
                v[i] -= 1;
            }
        }
    }

    fn compute_temporal_hash(timestamp: u64) -> u16 {
        // 纯时间戳哈希
        let mut hasher = XxHash64::with_seed(12345); // 独立种子
        timestamp.hash(&mut hasher);
        let h = hasher.finish();
        (h & 0xFFFF) as u16
    }

    /// 计算加权汉明距离相似度 (V2: 支持分区权重掩码)
    /// mask: 用于指定只关注哪些区域 (例如只关注时空区)
    pub fn similarity_weighted(a: u64, b: u64, mask: u64) -> f32 {
        let xor = (a ^ b) & mask;
        let dist = xor.count_ones();
        let total_bits = mask.count_ones();
        if total_bits == 0 { return 0.0; }
        1.0 - (dist as f32 / total_bits as f32)
    }
    
    /// 原始相似度接口
    pub fn similarity(a: u64, b: u64) -> f32 {
        // 默认全区匹配
        Self::similarity_weighted(a, b, 0xFFFFFFFFFFFFFFFF)
    }
}

// ============================================================================
// 2. 核心数据结构
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Feature, // 特征锚点（关键词、实体）
    Event,   // 事件总结节点（记忆主体）
}

#[derive(Clone, Debug)]
pub struct GraphEdge {
    pub target_node_id: i64,
    pub connection_strength: u16,
    pub edge_type: u8, // V2: 0=关联, 1=因果, 2=顺序, 3=对比
}

pub struct Node {
    pub id: i64,
    pub node_type: NodeType,
    pub content: String,       // 对于 Event 是总结，对于 Feature 是关键词
    pub fingerprint: u64,      // 语义指纹
    
    // V2 新增字段
    pub timestamp: u64,        // Unix 时间戳
    pub emotions: SmallVec<[u8; 8]>, // 情感矢量 (8维)
    pub prev_event: Option<i64>,     // 时序前驱
    pub next_event: Option<i64>,     // 时序后继
}

// ============================================================================
// 3. 高级实验引擎
// ============================================================================

pub struct ChaosStore {
    pub ids: Vec<i64>,
    pub fingerprints: Vec<ChaosFingerprint>,
    pub vectors: Vec<Vec<f16>>,
    pub id_to_index: AHashMap<i64, usize>,
}

impl ChaosStore {
    pub fn new() -> Self {
        Self {
            ids: Vec::new(),
            fingerprints: Vec::new(),
            vectors: Vec::new(),
            id_to_index: AHashMap::new(),
        }
    }

    pub fn add(&mut self, id: i64, fp: ChaosFingerprint, vec: Vec<f16>) {
        if !self.id_to_index.contains_key(&id) {
            let idx = self.ids.len();
            self.ids.push(id);
            self.fingerprints.push(fp);
            self.vectors.push(vec);
            self.id_to_index.insert(id, idx);
        }
    }
}

pub trait AsyncTaskInterface {
    fn schedule_maintenance(&self, context: &str);
}

pub struct MockAsyncTask;
impl AsyncTaskInterface for MockAsyncTask {
    fn schedule_maintenance(&self, _context: &str) {
        // Placeholder
    }
}

pub struct AdvancedEngine {
    pub nodes: AHashMap<i64, Node>,
    pub chaos_store: ChaosStore,
    pub graph: AHashMap<i64, SmallVec<[GraphEdge; 4]>>,
    
    // 第一套数据库：定义库 (Ontology)
    pub ontology_graph: AHashMap<i64, SmallVec<[GraphEdge; 4]>>,
    
    // 搜索辅助
    pub ac_matcher: Option<AhoCorasick>,
    pub feature_keywords: Vec<String>,
    pub keyword_to_node: AHashMap<String, i64>,
    
    // V2: 性能控制
    pub in_degrees: AHashMap<i64, u32>, // 预计算入度
    
    // V2: 时空索引 (Temporal Index) - 用于快速共振召回
    pub temporal_index: AHashMap<u16, Vec<i64>>,
    
    // V2: 情感索引 (Affective Index) - 用于情感共振
    pub affective_index: AHashMap<u8, Vec<i64>>,

    // V2: 异步接口
    pub async_task: Box<dyn AsyncTaskInterface + Send + Sync>,

    // 第四阶段: Candle 嵌入模型
    pub embedding_model: Option<CandleModel>,

    // GLiNER-X-Base: 实体类型 + 时间 span 提取 (替代硬编码 contains())
    pub gliner_engine: Option<GlinerEngine>,
}

impl AdvancedEngine {
    pub fn new() -> Self {
        Self {
            nodes: AHashMap::new(),
            chaos_store: ChaosStore::new(),
            graph: AHashMap::new(),
            ontology_graph: AHashMap::new(),
            ac_matcher: None,
            feature_keywords: Vec::new(),
            keyword_to_node: AHashMap::new(),
            in_degrees: AHashMap::new(),
            temporal_index: AHashMap::new(),
            affective_index: AHashMap::new(),
            async_task: Box::new(MockAsyncTask),
            embedding_model: None,
            gliner_engine: None,
        }
    }

    /// 添加特征节点
    pub fn add_feature(&mut self, id: i64, keyword: &str) {
        let keyword_lower = keyword.to_lowercase();
        
        // --- 停用词硬过滤 (双保险机制) ---
        // 包含中英文常见的虚词、介词、代词、助动词及连词
        let stopwords = [
            // 中文虚词
            "的", "是", "了", "在", "我", "你", "他", "她", "它", "们", "这", "那", "都", "和", "并", "且",
            "也", "就", "着", "吧", "吗", "呢", "啊", "呀", "呜", "哎", "哼", "呸", "喽",
            // 英文介词
            "a", "an", "the", "about", "above", "across", "after", "against", "along", "among", "around", "at", 
            "before", "behind", "below", "beneath", "beside", "between", "beyond", "but", "by", "despite", "down", 
            "during", "except", "for", "from", "in", "inside", "into", "like", "near", "of", "off", "on", "onto", 
            "out", "outside", "over", "past", "since", "through", "throughout", "till", "to", "toward", "under", 
            "underneath", "until", "up", "upon", "with", "within", "without",
            // 英文代词
            "i", "me", "my", "mine", "we", "us", "our", "ours", "you", "your", "yours", "he", "him", "his", 
            "she", "her", "hers", "it", "its", "they", "them", "their", "theirs", "this", "that", "these", "those", 
            "who", "whom", "whose", "which", "what", "each", "every", "either", "neither", "some", "any", "no", 
            "none", "both", "few", "many", "other", "another",
            // 英文助动词
            "am", "is", "are", "was", "were", "be", "being", "been", "have", "has", "had", "do", "does", "did", 
            "shall", "will", "should", "would", "may", "might", "must", "can", "could",
            // 英文连词及其他
            "and", "or", "so", "nor", "yet", "although", "because", "unless", "while", "where", "when", "how", "whether"
        ];
        if stopwords.contains(&keyword_lower.as_str()) {
            return;
        }

        let node = Node {
            id,
            node_type: NodeType::Feature,
            content: keyword_lower.clone(),
            fingerprint: SimHash::compute(&keyword_lower),
            timestamp: 0,
            emotions: SmallVec::new(),
            prev_event: None,
            next_event: None,
        };
        self.nodes.insert(id, node);
        self.feature_keywords.push(keyword_lower.clone());
        self.keyword_to_node.insert(keyword_lower, id);
    }

    /// 辅助：从文本中提取日期并转换为时间戳 (YYYY年MM月DD日)
    fn extract_timestamp(text: &str) -> u64 {
        // 简易解析器，查找 "20xx年xx月xx日"
        // 默认基准时间：2023-01-01 (1672531200)
        let default_ts = 1672531200;
        
        // 遍历所有 "年" 的出现位置
        for (year_idx, _) in text.match_indices("年") {
            if year_idx >= 4 && text.is_char_boundary(year_idx - 4) {
                if let Ok(year) = text[year_idx-4..year_idx].parse::<i32>() {
                    let mut day = 1;
                    
                    let rest = &text[year_idx+3..]; // 跳过 "年" (UTF-8 3字节)
                    
                    // 查找 "月"，且距离不应太远 (最多 5字节，容纳 " 12" 或 "1")
                    if let Some(month_idx) = rest.find("月") {
                        if month_idx <= 5 {
                            let m_str = rest[..month_idx].trim();
                            if let Ok(month) = m_str.parse::<i32>() {
                                
                                let rest_day = &rest[month_idx+3..];
                                // 查找 "日"，距离也不应太远
                                if let Some(day_idx) = rest_day.find("日") {
                                    if day_idx <= 5 {
                                        let d_str = rest_day[..day_idx].trim();
                                        if let Ok(d) = d_str.parse::<i32>() {
                                            day = d;
                                        }
                                    }
                                }
                                
                                // 简单转为 Unix 时间戳
                                let ts = (year as u64 - 1970) * 31536000 + (month as u64) * 2592000 + (day as u64) * 86400;
                                return ts;
                            }
                        }
                    }
                }
            }
        }
        default_ts
    }

    /// 混沌向量化接口：将文本自动转换为 512 维 f16 向量和 512-bit ChaosFingerprint 指纹
    pub fn calculate_chaos(&self, text: &str) -> Option<(ChaosFingerprint, Vec<f16>)> {
        let model = self.embedding_model.as_ref()?;
        
        let mut weighted_ranges = Vec::new();
        if let Some(matcher) = &self.ac_matcher {
            for mat in matcher.find_iter(&text.to_lowercase()) {
                weighted_ranges.push((mat.start(), mat.end(), 5.0));
            }
        }

        if let Some(vec_f32) = model.vectorize_weighted(text, &weighted_ranges) {
            let chaos_vector: Vec<f16> = vec_f32.iter().map(|&x| f16::from_f32(x)).collect();
            let chaos_fingerprint = StorageEngine::quantize_vector(&chaos_vector);
            Some((chaos_fingerprint, chaos_vector))
        } else {
            None
        }
    }

    /// 添加事件节点
    pub fn add_event(&mut self, id: i64, summary: &str, chaos_fp: Option<ChaosFingerprint>, chaos_vec: Option<Vec<f16>>) {
        // 自动提取时间戳 (原始解析器)
        let mut timestamp = Self::extract_timestamp(summary);

        // 自动提取情感
        let emotion_val = SimHash::extract_emotion(summary);

        // GLiNER 增强: 提取实体类型 + 补充时间
        let type_val = if let Some(gliner) = &self.gliner_engine {
            let (type_entities, time_entities) = gliner.extract_all(summary);
            
            // 如果原始解析器未提取到时间，用 GLiNER 补充
            if timestamp == 0 && !time_entities.is_empty() {
                let ref_time = instant::Instant::now().elapsed().as_secs(); // 近似
                timestamp = gliner_ner::best_timestamp(&time_entities, ref_time);
            }
            
            gliner_ner::best_type_val(&type_entities)
        } else {
            SimHash::TYPE_UNKNOWN
        };

        // V2: 在入库时自动进行时空/情感/类型特征提取 (自动打标)
        let fingerprint = SimHash::compute_multimodal(summary, timestamp, emotion_val, type_val);

        // V3 第四阶段: 自动向量化 (混沌向量)
        let mut chaos_fingerprint = chaos_fp.unwrap_or(ChaosFingerprint::default());
        let mut chaos_vector = chaos_vec.unwrap_or_default();

        if chaos_fingerprint == ChaosFingerprint::default() && chaos_vector.is_empty() {
            if let Some((fp, vec)) = self.calculate_chaos(summary) {
                chaos_fingerprint = fp;
                chaos_vector = vec;
            }
        }
        
        // 填充情感向量
        let mut emotions = SmallVec::new();
        for i in 0..8 {
            if (emotion_val & (1 << i)) != 0 {
                emotions.push(1 << i);
            }
        }
        
        let node = Node {
            id,
            node_type: NodeType::Event,
            content: summary.to_string(),
            fingerprint,
            timestamp, 
            emotions,
            prev_event: None,
            next_event: None,
        };
        self.nodes.insert(id, node);
        
        // SoA 存储
        if chaos_fingerprint != ChaosFingerprint::default() || !chaos_vector.is_empty() {
             self.chaos_store.add(id, chaos_fingerprint, chaos_vector);
        }

        // V2: 更新倒排索引 (Inverted Indexes) 用于快速召回
        // 1. 时空索引
        if (fingerprint & SimHash::MASK_TEMPORAL) != 0 {
            let st_hash = ((fingerprint & SimHash::MASK_TEMPORAL) >> 32) as u16;
            self.temporal_index.entry(st_hash).or_default().push(id);
        }

        // 2. 情感索引
        if (fingerprint & SimHash::MASK_AFFECTIVE) != 0 {
            let emotion_hash = ((fingerprint & SimHash::MASK_AFFECTIVE) >> 48) as u8;
            for i in 0..8 {
                if (emotion_hash & (1 << i)) != 0 {
                    self.affective_index.entry(1 << i).or_default().push(id);
                }
            }
        }
    }

    /// 建立关联 (V2: 增加重复边检测与强度更新逻辑)
    pub fn add_edge(&mut self, src: i64, tgt: i64, weight: f32) {
        let quantized = (weight.clamp(0.0, 1.0) * 65535.0) as u16;
        let edges = self.graph.entry(src).or_default();
        
        if let Some(edge) = edges.iter_mut().find(|e| e.target_node_id == tgt) {
            // 如果边已存在，更新为较大的强度值 (模拟记忆增强)
            if quantized > edge.connection_strength {
                edge.connection_strength = quantized;
            }
        } else {
            edges.push(GraphEdge {
                target_node_id: tgt,
                connection_strength: quantized,
                edge_type: 0,
            });
        }
    }

    /// 添加定义库关联 (第一套数据库)
    /// relation_type: "equality" | "inhibition" | "representation"
    pub fn maintain_ontology(&mut self, source: &str, target: &str, relation_type: &str, strength: f32) {
        println!("🤖 [LLM Maintenance] 发现新关联: {} -> {} (type: {}, strength: {})", 
                 source, target, relation_type, strength);
        
        let src_id = self.get_or_create_feature(source);
        let tgt_id = self.get_or_create_feature(target);
        
        let strength_u16 = (strength * 65535.0) as u16;
        
        // 确定边类型 (简化为三种核心逻辑)
        let edge_type = match relation_type.to_lowercase().as_str() {
            "equality" | "equal" => 1, // SimHash::EDGE_EQUALITY (暂时硬编码以修复编译错误)
            "inhibition" | "conflict" => 255, // SimHash::EDGE_INHIBITION
            _ => 0, // SimHash::EDGE_REPRESENTATION
        };

        // 处理正向边
        {
            let edges = self.ontology_graph.entry(src_id).or_default();
            if let Some(edge) = edges.iter_mut().find(|e| e.target_node_id == tgt_id) {
                // [LTD 机制] 被动强化 (Hebbian Learning)
                edge.connection_strength = edge.connection_strength.saturating_add(strength_u16 / 2).max(strength_u16);
                // 更新类型
                edge.edge_type = edge_type;
            } else {
                edges.push(GraphEdge {
                    target_node_id: tgt_id,
                    connection_strength: strength_u16,
                    edge_type,
                });
            }
        }
        
        // 处理反向边
        // 1. Equality (Type 1): 强制双向，表示 A==B 且 B==A
        // 2. Inhibition (Type 255): 强制双向，表示 A互斥B 且 B互斥A
        // 3. Representation (Type 0): 默认单向 (Directed)，因为"看到B想到A"不代表"看到A一定想到B"
        //    (除非业务层显式要求双向，否则底层只存单向)
        if edge_type == 1 || edge_type == 255 {
            let rev_edges = self.ontology_graph.entry(tgt_id).or_default();
            if let Some(edge) = rev_edges.iter_mut().find(|e| e.target_node_id == src_id) {
                // [LTD 机制] 被动强化
                edge.connection_strength = edge.connection_strength.saturating_add(strength_u16 / 2).max(strength_u16);
                edge.edge_type = edge_type;
            } else {
                rev_edges.push(GraphEdge {
                    target_node_id: src_id,
                    connection_strength: strength_u16,
                    edge_type,
                });
            }
        }
    }

    // ========================================================================
    // 动态剪枝 (LTD: Long-Term Depression)
    // ========================================================================

    /// 执行全局衰减与物理剪枝
    /// decay_rate: 衰减比率 (0.0 - 1.0)，建议 0.95
    /// threshold: 剪枝阈值 (0 - 65535)，建议 3276 (0.05)
    pub fn apply_global_decay_and_pruning(&mut self, decay_rate: f32, threshold: u16) -> usize {
        let mut pruned_count = 0;
        
        // 遍历整个 Ontology 图谱
        for edges in self.ontology_graph.values_mut() {
            // 1. 全局熵增 (Entropy Increase)
            for edge in edges.iter_mut() {
                let current = edge.connection_strength as f32;
                edge.connection_strength = (current * decay_rate) as u16;
            }
            
            // 2. 物理断裂 (Pruning)
            let before_len = edges.len();
            edges.retain(|e| e.connection_strength > threshold);
            let after_len = edges.len();
            
            pruned_count += before_len - after_len;
        }
        
        if pruned_count > 0 {
            println!("[PEDSA Memory] Pruning executed: {} synapses disconnected.", pruned_count);
        }
        
        pruned_count
    }

    fn get_or_create_feature(&mut self, word: &str) -> i64 {
        let word_lower = word.to_lowercase();
        
        // 停用词检查 (同步 add_feature 中的列表)
        let stopwords = [
            // 中文虚词
            "的", "是", "了", "在", "我", "你", "他", "她", "它", "们", "这", "那", "都", "和", "并", "且",
            "也", "就", "着", "吧", "吗", "呢", "啊", "呀", "呜", "哎", "哼", "呸", "喽",
            // 英语介词
            "a", "an", "the", "about", "above", "across", "after", "against", "along", "among", "around", "at", 
            "before", "behind", "below", "beneath", "beside", "between", "beyond", "but", "by", "despite", "down", 
            "during", "except", "for", "from", "in", "inside", "into", "like", "near", "of", "off", "on", "onto", 
            "out", "outside", "over", "past", "since", "through", "throughout", "till", "to", "toward", "under", 
            "underneath", "until", "up", "upon", "with", "within", "without",
            // 英语代词
            "i", "me", "my", "mine", "we", "us", "our", "ours", "you", "your", "yours", "he", "him", "his", 
            "she", "her", "hers", "it", "its", "they", "them", "their", "theirs", "this", "that", "these", "those", 
            "who", "whom", "whose", "which", "what", "each", "every", "either", "neither", "some", "any", "no", 
            "none", "both", "few", "many", "other", "another",
            // 英语助动词
            "am", "is", "are", "was", "were", "be", "being", "been", "have", "has", "had", "do", "does", "did", 
            "shall", "will", "should", "would", "may", "might", "must", "can", "could",
            // 英语连词及其他
            "and", "or", "so", "nor", "yet", "although", "because", "unless", "while", "where", "when", "how", "whether"
        ];
        if stopwords.contains(&word_lower.as_str()) {
            return -1; // 返回非法 ID 表示该词被屏蔽
        }

        if let Some(&id) = self.keyword_to_node.get(&word_lower) {
            id
        } else {
            let mut s = XxHash64::with_seed(0);
            word_lower.hash(&mut s);
            let id = (s.finish() as i64).abs();
            self.add_feature(id, &word_lower);
            id
        }
    }

    /// 建立双向时序链表 (Temporal Backbone)
    pub fn build_temporal_backbone(&mut self) {
        println!("⏳ 正在构建时序脊梁 (Temporal Backbone)...");
        
        // 1. 收集所有 Event 节点并按时间戳排序
        let mut events: Vec<(i64, u64)> = self.nodes.values()
            .filter(|n| n.node_type == NodeType::Event)
            .map(|n| (n.id, n.timestamp))
            .collect();
        
        // 如果没有时间戳，暂时用 ID 模拟顺序（仅供测试）
        // 在真实场景中，timestamp 应该是必填的
        events.sort_by(|a, b| {
            if a.1 != b.1 {
                a.1.cmp(&b.1)
            } else {
                a.0.cmp(&b.0) // 时间戳相同则按 ID 排序
            }
        });

        // 2. 串联双向链表
        for i in 0..events.len() {
            let (curr_id, _) = events[i];
            
            if i > 0 {
                let (prev_id, _) = events[i-1];
                if let Some(node) = self.nodes.get_mut(&curr_id) {
                    node.prev_event = Some(prev_id);
                }
            }
            
            if i < events.len() - 1 {
                let (next_id, _) = events[i+1];
                if let Some(node) = self.nodes.get_mut(&curr_id) {
                    node.next_event = Some(next_id);
                }
            }
        }
        println!("✅ 时序脊梁构建完成，已串联 {} 个事件节点。", events.len());
    }

    /// 加载标准测试数据集
    pub fn load_standard_data(&mut self) {
        println!("📦 正在注入硬核测试数据...");
        let mut all_events = Vec::new();
        let mut all_edges = Vec::new();

        let (e1, d1) = get_tech_domain_data();
        all_events.extend(e1); all_edges.extend(d1);

        let (e2, d2) = get_social_domain_data();
        all_events.extend(e2); all_edges.extend(d2);

        let (e3, d3) = get_history_domain_data();
        all_events.extend(e3); all_edges.extend(d3);

        let (e4, d4) = get_value_domain_data();
        all_events.extend(e4); all_edges.extend(d4);

        let (e5, d5) = get_daily_domain_data();
        all_events.extend(e5); all_edges.extend(d5);

        let (e6, d6) = get_timeline_domain_data();
        all_events.extend(e6); all_edges.extend(d6);
        
        // 注入节点与特征
        for ev in all_events {
            self.add_event(ev.id, ev.summary, ev.chaos_fingerprint, ev.chaos_vector);
            for feature in ev.features {
                let feature_lower = feature.to_lowercase();
                let mut s = XxHash64::with_seed(0);
                feature_lower.hash(&mut s);
                let feat_id = (s.finish() as i64).abs();
                
                self.add_feature(feat_id, &feature_lower);
                self.add_edge(feat_id, ev.id, 1.0);
            }
        }

        // 2. 加载定义库数据 (Ontology)
        println!("📚 正在注入定义库 (Ontology) 数据...");
        let ontology_edges = get_ontology_data();
        for edge in ontology_edges {
            // V3.5: 使用带关系类型的 maintain_ontology
            let relation_type = if edge.is_equality {
                "equality"
            } else if edge.is_inhibition {
                "inhibition"
            } else {
                "representation"
            };
            self.maintain_ontology(edge.src, edge.tgt, relation_type, edge.weight);
        }

        // 3. 注入逻辑边
        for edge in all_edges {
            self.add_edge(edge.src, edge.tgt, edge.weight);
        }

        // 4. 手动建立一些跨领域的“隐秘关联”
        self.add_edge(205, 100, 0.6); // 1k star (205) 与 PeroCore 重构 (100)
        self.add_edge(200, 302, 0.4); // 面试被套方案 (200) 与 PEASE 算法灵感 (302)

        // V2: 构建时序脊梁
        self.build_temporal_backbone();
    }

    /// 编译 AC 自动机
    pub fn compile(&mut self) {
        // 只对 Feature 节点编译 AC 自动机
        let mut keywords: Vec<_> = self.nodes.values()
            .filter(|n| n.node_type == NodeType::Feature)
            .map(|n| n.content.clone())
            .collect();
        
        // V2: 关键优化 - 按长度降序排序，确保优先匹配长词 (如 "分布式编译" 优于 "分布式")
        keywords.sort_by(|a, b| b.len().cmp(&a.len()));

        if !keywords.is_empty() {
            self.ac_matcher = Some(AhoCorasickBuilder::new()
                .match_kind(MatchKind::LeftmostLongest)
                .build(&keywords)
                .unwrap());
            self.feature_keywords = keywords;
        }

        // V2: 计算节点入度 (In-degree) 以用于反向抑制
        self.in_degrees.clear();
        // 统计 Memory Graph
        for edges in self.graph.values() {
            for edge in edges {
                *self.in_degrees.entry(edge.target_node_id).or_default() += 1;
            }
        }
        // 统计 Ontology Graph
        for edges in self.ontology_graph.values() {
            for edge in edges {
                *self.in_degrees.entry(edge.target_node_id).or_default() += 1;
            }
        }

        // V2: 构建时空索引 (Spatio-Temporal Index) 与 情感索引 (Affective Index)
        self.temporal_index.clear();
        self.affective_index.clear();

        for node in self.nodes.values() {
            if node.node_type == NodeType::Event {
                // 时空索引
                let st_hash = ((node.fingerprint & SimHash::MASK_TEMPORAL) >> 32) as u16;
                if st_hash != 0 {
                    self.temporal_index.entry(st_hash).or_default().push(node.id);
                }

                // 情感索引
                let emotion_hash = ((node.fingerprint & SimHash::MASK_AFFECTIVE) >> 48) as u8;
                if emotion_hash != 0 {
                    // 对于每个设置了的位，都加入到对应的索引桶中 (支持混合情感)
                    for i in 0..8 {
                        if (emotion_hash & (1 << i)) != 0 {
                            self.affective_index.entry(1 << i).or_default().push(node.id);
                        }
                    }
                }
            }
        }

        // GLiNER-X-Base 初始化
        match GlinerEngine::new("models/gliner-x-base") {
            Ok(mut engine) => {
                // 将 Feature 节点的关键词加入 jieba 自定义词典
                let mut custom_count = 0;
                for node in self.nodes.values() {
                    if node.node_type == NodeType::Feature && node.content.len() >= 2 {
                        engine.add_custom_word(&node.content);
                        custom_count += 1;
                    }
                }
                println!("🏷️  GLiNER-X-Base 已加载 (ONNX Runtime), {} 个自定义词", custom_count);
                self.gliner_engine = Some(engine);
            }
            Err(e) => {
                println!("⚠️  GLiNER 未加载: {}, 降级为关键词匹配", e);
            }
        }

        println!("🚀 引擎编译完成：{} 个特征锚点, {} 个总节点, {} 个时空桶, {} 个情感维度", 
            self.feature_keywords.len(), self.nodes.len(), self.temporal_index.len(), self.affective_index.len());
    }

    /// 大规模压力测试数据生成器
    pub fn load_million_test_data(&mut self, node_count: usize) {
        println!("🏗️ 正在生成 {} 级大规模合成数据...", node_count);
        let start = Instant::now();

        // 预分配内存以应对千万级别的压力，减少重新分配导致的性能抖动和内存碎片
        self.nodes.reserve(node_count + 50000);
        self.graph.reserve(node_count + 50000);
        self.keyword_to_node.reserve(50000);
        self.feature_keywords.reserve(50000);
        
        // 1. 生成特征节点 (固定 50,000 个，更符合真实 Ontology 规模)
        let feature_count = 50_000;
        for i in 0..feature_count {
            let id = i as i64 + 1_000_000_000;
            let kw = format!("feat_{}", i);
            self.add_feature(id, &kw);
        }

        // 2. 生成事件节点
        let event_count = node_count;
        for i in 0..event_count {
            let id = i as i64 + 2_000_000_000;
            let summary = format!("这是一个模拟的事件总结节点，编号为 {}，用于进行规模压力测试。PEDSA 算法应当在这种规模下依然保持极高的检索效率。", i);
            self.add_event(id, &summary, None, None);
            
            // 每个事件随机关联 1-3 个特征
            let feat_idx = i % feature_count;
            let feat_id = feat_idx as i64 + 1_000_000_000;
            self.add_edge(feat_id, id, 1.0);
            
            if i % 2 == 0 {
                let feat_id_2 = (i * 7 % feature_count) as i64 + 1_000_000_000;
                self.add_edge(feat_id_2, id, 0.8);
            }
        }

        println!("✅ 数据加载完成，耗时: {:?}", start.elapsed());
    }

    /// 执行多级检索 (V2: 增加能量控制机制 + 分区时空共振)
    /// 第四阶段：双轨检索（理性 + 混沌）
    /// 
    /// # 参数
    /// * `query` - 查询字符串。
    /// * `ref_time` - 用于相对时间解析的参考时间戳。
    /// * `chaos_level` - 0.0 到 1.0 之间的浮点数。
    ///   - 0.0: 纯理性检索（确定性）。
    ///   - 1.0: 纯混沌检索（高随机性/创造性）。
    ///   - 中间值则混合两者的得分。
    pub fn retrieve(&self, query: &str, ref_time: u64, chaos_level: f32) -> Vec<(i64, f32)> {
        let mut activated_keywords = AHashMap::new();
        let query_lower = query.to_lowercase();

        // V3: GLiNER 驱动的指纹生成 (回退到硬编码 compute_for_query)
        let query_fp = if let Some(gliner) = &self.gliner_engine {
            let (type_entities, time_entities) = gliner.extract_all(&query_lower);
            
            // TYPE: 取最高置信度的类型
            let type_val = gliner_ner::best_type_val(&type_entities);
            
            // TEMPORAL: 取最高置信度且可解析的时间 span
            let timestamp = gliner_ner::best_timestamp(&time_entities, ref_time);
            
            // AFFECTIVE: 保持现有关键词匹配 (GLiNER 不适合情感分类)
            let emotion = SimHash::extract_emotion(&query_lower);
            
            SimHash::compute_multimodal(&query_lower, timestamp, emotion, type_val)
        } else {
            SimHash::compute_for_query(&query_lower, ref_time)
        };

        // --- Step 1: 特征共振 (AC Matcher) - 极快 ---
        if let Some(matcher) = &self.ac_matcher {
            for mat in matcher.find_iter(&query_lower) {
                let kw = &self.feature_keywords[mat.pattern()];
                if let Some(&node_id) = self.keyword_to_node.get(kw) {
                    activated_keywords.insert(node_id, 1.0);
                }
            }
        }

        // --- Step 1.5: 时间共振 (Temporal Resonance) ---
        // 如果 Query 包含时间信息，直接从索引中召回候选节点 (绕过语义匹配)
        if (query_fp & SimHash::MASK_TEMPORAL) != 0 {
            let st_hash = ((query_fp & SimHash::MASK_TEMPORAL) >> 32) as u16;
            if let Some(candidates) = self.temporal_index.get(&st_hash) {
                // 将这些候选节点加入初始激活集合
                // 注意：这些通常是 Event 节点，它们将直接作为种子进入后续流程
                for &id in candidates {
                    let entry = activated_keywords.entry(id).or_insert(0.0);
                    // 初始共振能量设为 0.6 (低于完全匹配的 1.0)
                    if *entry < 0.6 { *entry = 0.6; }
                }
            }
        }

        // --- Step 1.6: 情感共振 (Affective Resonance) ---
        // 如果 Query 包含情感信息，从情感索引中召回候选节点
        if (query_fp & SimHash::MASK_AFFECTIVE) != 0 {
            let emotion_hash = ((query_fp & SimHash::MASK_AFFECTIVE) >> 48) as u8;
            for i in 0..8 {
                if (emotion_hash & (1 << i)) != 0 {
                    if let Some(candidates) = self.affective_index.get(&(1 << i)) {
                         for &id in candidates {
                            let entry = activated_keywords.entry(id).or_insert(0.0);
                            // 情感共振能量设为 0.7 (比较强烈，因为是内心的直接投射)
                            if *entry < 0.7 { *entry = 0.7; }
                        }
                    }
                }
            }
        }

        // --- Step 1.9: BM25 暴力兜底 (Cold-Start Fallback) ---
        // 当 AC 自动机 + 时间共振 + 情感共振 全部未命中时触发
        // 场景: 图谱尚未完善，用户输入没有任何词被 Ontology 覆盖
        if activated_keywords.is_empty() {
            // 分词: 优先 jieba (来自 GLiNER)，否则用字符 bigram
            let query_tokens: Vec<String> = if let Some(gliner) = &self.gliner_engine {
                gliner.jieba.cut(&query_lower, false)
                    .into_iter()
                    .filter(|t| t.len() >= 2) // 过滤单字符
                    .map(|t| t.to_string())
                    .collect()
            } else {
                // 字符 bigram 回退
                let chars: Vec<char> = query_lower.chars().collect();
                chars.windows(2).map(|w| w.iter().collect::<String>()).collect()
            };

            if !query_tokens.is_empty() {
                // BM25 参数
                let k1: f32 = 1.2;
                let b: f32 = 0.75;
                
                // 预计算: 文档总数 & 平均长度
                let event_nodes: Vec<&Node> = self.nodes.values()
                    .filter(|n| n.node_type == NodeType::Event)
                    .collect();
                let n_docs = event_nodes.len() as f32;
                if n_docs > 0.0 {
                    let avg_dl: f32 = event_nodes.iter()
                        .map(|n| n.content.len() as f32)
                        .sum::<f32>() / n_docs;

                    // IDF: 统计每个 query token 出现在多少文档中
                    let mut df: AHashMap<&str, u32> = AHashMap::new();
                    for token in &query_tokens {
                        for node in &event_nodes {
                            if node.content.contains(token.as_str()) {
                                *df.entry(token.as_str()).or_default() += 1;
                            }
                        }
                    }

                    // 对每个 Event 节点打分
                    let mut bm25_scores: Vec<(i64, f32)> = Vec::new();
                    for node in &event_nodes {
                        let dl = node.content.len() as f32;
                        let mut score = 0.0f32;
                        
                        for token in &query_tokens {
                            let tf = node.content.matches(token.as_str()).count() as f32;
                            if tf == 0.0 { continue; }
                            
                            let doc_freq = *df.get(token.as_str()).unwrap_or(&1) as f32;
                            let idf = ((n_docs - doc_freq + 0.5) / (doc_freq + 0.5) + 1.0).ln();
                            
                            let tf_norm = (tf * (k1 + 1.0)) / (tf + k1 * (1.0 - b + b * dl / avg_dl));
                            score += idf * tf_norm;
                        }
                        
                        if score > 0.0 {
                            bm25_scores.push((node.id, score));
                        }
                    }
                    
                    // 归一化 + 取 Top-20
                    if !bm25_scores.is_empty() {
                        bm25_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                        let max_score = bm25_scores[0].1;
                        
                        for (id, score) in bm25_scores.iter().take(20) {
                            let normalized = score / max_score * 0.4; // BM25 信号上限 0.4 (弱于精确匹配)
                            activated_keywords.insert(*id, normalized);
                        }
                    }
                }
            }
        }

        // --- Step 2: 第一数据库 (Ontology 定义库) 扩散 ---
        let mut ontology_expanded = activated_keywords.clone();
        for (&node_id, &score) in &activated_keywords {
            if let Some(neighbors) = self.ontology_graph.get(&node_id) {
                for edge in neighbors {
                    let weight = edge.connection_strength as f32 / 65535.0;
                    
                    // V2: 反向抑制 (Inverse Inhibition) - 降低泛化词权重
                    let degree = self.in_degrees.get(&edge.target_node_id).unwrap_or(&1);
                    // log10(1)=0 -> 1.0; log10(10)=1 -> 0.5; log10(100)=2 -> 0.33
                    let inhibition_factor = 1.0 / (1.0 + (*degree as f32).log10()); 
                    
                    // V3.5: 类型化边逻辑
                    // 1. EQUALITY (1): 零损耗，无视反向抑制，能量直接传递 (max)
                    // 2. INHIBITION (255): 负能量扣减
                    // 3. REPRESENTATION (0): 正常衰减
                    
                    if edge.edge_type == SimHash::EDGE_EQUALITY {
                        // 等价传递：直接取源节点能量，不打折
                        let entry = ontology_expanded.entry(edge.target_node_id).or_insert(0.0);
                        if score > *entry {
                             *entry = score;
                        }
                        continue;
                    }
                    
                    // 计算基础能量 (共现增益: log 衰减 + 反向抑制)
                    // log(1 + w*65535) / log(65536) 将线性权重映射为对数曲线
                    // 效果: 高频边的边际收益递减，低频边获得更多相对能量
                    let log_weight = (1.0 + weight * 65535.0).ln() / (65536.0_f32).ln();
                    let energy = score * log_weight * 0.95 * inhibition_factor;
                    
                    if edge.edge_type == SimHash::EDGE_INHIBITION {
                        // 抑制传递：扣减能量
                        // 注意：如果目标节点尚未激活 (0.0)，扣减后为负，之后会被截断
                        let entry = ontology_expanded.entry(edge.target_node_id).or_insert(0.0);
                        *entry -= energy; 
                    } else {
                        // 普通传递
                        // V2: 硬阈值剪枝 (Hard Squelch)
                        if energy < 0.05 { continue; }
                        
                        let entry = ontology_expanded.entry(edge.target_node_id).or_insert(0.0);
                        *entry = (*entry).max(energy);
                    }
                }
            }
        }

        // --- Step 3: 能量归一化 (Energy Normalization) ---
        // 防止扩散到 Memory 库前能量过大
        let total_energy: f32 = ontology_expanded.values().sum();
        if total_energy > 10.0 {
            let factor = 10.0 / total_energy;
            for val in ontology_expanded.values_mut() {
                *val *= factor;
            }
        }

        // --- Step 4: 第二数据库 (Memory 记忆库) 扩散 ---
        let final_scores = ontology_expanded.clone();
        let decay = 0.85; // 提高衰减系数，增加信号传播距离
        let layer_limit = 5000; 

        // 侧向抑制：选出能量最高的 Top-K 种子进行扩散
        let mut seeds: Vec<(&i64, &f32)> = ontology_expanded.iter().collect();
        // 排序
        seeds.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
        // 截断 (Lateral Inhibition)
        if seeds.len() > layer_limit {
            seeds.truncate(layer_limit);
        }

        let increments: AHashMap<i64, f32> = seeds
            .into_par_iter()
            .fold(
                || AHashMap::new(),
                |mut acc, (&node_id, &score)| {
                    if let Some(neighbors) = self.graph.get(&node_id) {
                        for edge in neighbors {
                            let weight = edge.connection_strength as f32 / 65535.0;
                            
                            // V2: 反向抑制 (Memory 层)
                            let degree = self.in_degrees.get(&edge.target_node_id).unwrap_or(&1);
                            let inhibition_factor = 1.0 / (1.0 + (*degree as f32).log10());

                            // 共现增益: log 衰减 (与 Ontology 层一致)
                            let log_weight = (1.0 + weight * 65535.0).ln() / (65536.0_f32).ln();
                            let energy = score * log_weight * decay * inhibition_factor;
                            
                            // Memory 层阈值稍低，保留更多细节
                            if energy < 0.01 { continue; } 

                            *acc.entry(edge.target_node_id).or_default() += energy;
                        }
                    }
                    acc
                },
            )
            .reduce(
                || AHashMap::new(),
                |mut m1, m2| {
                    for (k, v) in m2 { *m1.entry(k).or_default() += v; }
                    m1
                },
            );

        // --- Step 5: 结果整合与局部 SimHash 细化 ---
        let mut results_map = final_scores;
        for (id, energy) in increments {
            *results_map.entry(id).or_default() += energy;
        }

        let mut results: Vec<_> = results_map
            .into_iter()
            .filter(|(id, _)| self.nodes.get(id).map_or(false, |n| n.node_type == NodeType::Event))
            .collect();

        // 局部细化：只对初步排序前 50 的结果进行 SimHash 修正
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        for i in 0..results.len().min(50) {
            let (id, score) = &mut results[i];
            if let Some(node) = self.nodes.get(id) {
                // V2: 分区多模态共振逻辑
                // 1. 语义共振 (基础)
                let semantic_sim = SimHash::similarity_weighted(query_fp, node.fingerprint, SimHash::MASK_SEMANTIC);
                let mut resonance_boost = semantic_sim * 0.6; // 显著提升语义共振权重
                
                // 2. 时间共振 (Temporal Resonance)
                // 只有当 Query 显式包含时空信息时 (mask 区域非零)，才进行加权
                if (query_fp & SimHash::MASK_TEMPORAL) != 0 {
                    let temporal_sim = SimHash::similarity_weighted(query_fp, node.fingerprint, SimHash::MASK_TEMPORAL);
                    // 时空匹配给予高权重 (0.5)，模拟“瞬间回忆”
                    resonance_boost += temporal_sim * 0.5;
                }

                // 3. 情感共鸣 (Affective Resonance) - Bitwise AND
                if (query_fp & SimHash::MASK_AFFECTIVE) != 0 {
                    let query_emotions = (query_fp & SimHash::MASK_AFFECTIVE) >> 48;
                    let node_emotions = (node.fingerprint & SimHash::MASK_AFFECTIVE) >> 48;
                    
                    // 位运算共振：只要有共同的情感位被激活，就产生强烈共鸣
                    if (query_emotions & node_emotions) != 0 {
                        resonance_boost += 0.6; 
                    }
                }

                // 4. 类型对齐 (Entity Type Alignment)
                if (query_fp & SimHash::MASK_TYPE) != 0 {
                    let type_sim = SimHash::similarity_weighted(query_fp, node.fingerprint, SimHash::MASK_TYPE);
                    // 类型匹配给予极高的修正权重 (0.8)，因为类型不对通常意味着完全无关
                    resonance_boost += type_sim * 0.8;
                }

                // 5. 艾宾浩斯记忆衰减 (Ebbinghaus Decay)
                // Formula: Energy = Base * e^(-t/tau)
                // 使用传入的 ref_time 作为衰减基准时间 (如果为 0 则默认不衰减)
                let current_decay_time = if ref_time > 0 { ref_time } else { 1777593600 }; 
                let tau = 31536000.0; // 延长记忆半衰期
                
                if node.timestamp > 0 && node.timestamp < current_decay_time {
                    let delta_t = (current_decay_time - node.timestamp) as f32;
                    let decay_factor = (-delta_t / tau).exp();
                    
                    // 降低衰减总权重：限制衰减系数最低为 0.8 (旧记忆最多损失 20% 能量)
                    let final_decay = decay_factor.max(0.8);
                    *score *= final_decay;
                }

                *score += resonance_boost;
            }
        }

        // --- 第四阶段：混沌激活 (双轨并行) ---
        if chaos_level > 0.0 {
            if let Some((query_fp, query_vec_f16)) = self.calculate_chaos(query) {
                let mut combined_results = AHashMap::new();
                
                // 将理性检索结果存入 map (按 1 - chaos_level 加权)
                for (id, score) in results.iter() {
                    combined_results.insert(*id, *score * (1.0 - chaos_level));
                }

                // --- 1. L1 粗排 (1-bit 量化) ---
                // 计算所有事件节点的汉明距离
                // 保留前 5000 个候选者
                
                // SoA 扫描
                let mut candidates: Vec<(usize, u32)> = Vec::with_capacity(self.chaos_store.ids.len() / 10);

                for (idx, &node_fp) in self.chaos_store.fingerprints.iter().enumerate() {
                    // 汉明距离：异或 -> 位计数 (不同位的数量)
                    let distance = query_fp.hamming_distance(&node_fp);
                    
                    // 阈值剪枝：最大距离 256 (总共 512 位) 意味着相关性几乎为 0
                    if distance < 256 {
                        candidates.push((idx, distance));
                    }
                }

                // 按距离排序 (升序)
                candidates.sort_unstable_by_key(|k| k.1);
                
                // 截取前 5000 个
                if candidates.len() > 5000 {
                    candidates.truncate(5000);
                }

                // --- 2. L2 精排 (f16 余弦相似度) ---
                let q_norm: f32 = query_vec_f16.iter().map(|x| x.to_f32().powi(2)).sum::<f32>().sqrt();
                
                for (idx, _dist) in candidates {
                    let node_id = self.chaos_store.ids[idx];
                    let chaos_vector = &self.chaos_store.vectors[idx];

                    if !chaos_vector.is_empty() {
                        let dot: f32 = query_vec_f16.iter().zip(chaos_vector).map(|(a, b)| a.to_f32() * b.to_f32()).sum();
                        let n_norm: f32 = chaos_vector.iter().map(|x| x.to_f32().powi(2)).sum::<f32>().sqrt();
                        
                        if q_norm > 0.0 && n_norm > 0.0 {
                            let sim = dot / (q_norm * n_norm);
                            
                            // 非线性激活 (阈值 > 0.6, 最大系数 0.15)
                            if sim > 0.6 {
                                let normalized = (sim - 0.6) / 0.4;
                                let chaos_score = normalized * 0.15;
                                let weighted_chaos = chaos_score * chaos_level;
                                
                                *combined_results.entry(node_id).or_default() += weighted_chaos;
                            }
                        }
                    }
                }
                
                // 转换回排序后的向量
                let mut final_results: Vec<_> = combined_results.into_iter().collect();
                final_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                return final_results;
            }
        }
        
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // --- Step 6: DPP 贪心多样性采样 (Determinantal Point Process) ---
        // 防止同一 Feature 关联的多个 Event 霸占 Top-K
        // 参考: Kulesza & Taskar, 2012, "Determinantal Point Processes for Machine Learning"
        if results.len() > 10 {
            let dpp_candidates = results.len().min(50);
            let dpp_select = 10; // 最终返回数
            let selected_indices = self.dpp_greedy_select(&results[..dpp_candidates], dpp_select);
            
            // 重组: DPP 选中的 + 剩余的 (保持 DPP 顺序优先)
            let mut dpp_results: Vec<(i64, f32)> = selected_indices.iter()
                .map(|&i| results[i])
                .collect();
            // 附加 DPP 候选范围外的结果
            for item in results.iter().skip(dpp_candidates) {
                dpp_results.push(*item);
            }
            return dpp_results;
        }

        results
    }

    /// DPP 贪心多样性采样
    /// 基于 SimHash 汉明距离构建相似度核矩阵，贪心最大化 log-det
    fn dpp_greedy_select(&self, candidates: &[(i64, f32)], k: usize) -> Vec<usize> {
        let n = candidates.len();
        if n <= k {
            return (0..n).collect();
        }

        // 1. 构建质量因子 q_i = score^0.8 (略微压缩分数差距)
        let quality: Vec<f32> = candidates.iter()
            .map(|(_, s)| s.max(1e-10).powf(0.8))
            .collect();

        // 2. 构建相似度矩阵 S_ij (基于 SimHash 汉明距离)
        //    S_ij = 1 - hamming(fp_i, fp_j) / 32  (语义区 32 位)
        let fingerprints: Vec<u64> = candidates.iter()
            .map(|(id, _)| self.nodes.get(id).map_or(0, |n| n.fingerprint))
            .collect();

        // 3. L-ensemble 核矩阵: L_ij = q_i * S_ij * q_j
        //    对角线 L_ii = q_i² (自相似度 = 1)
        let mut diag: Vec<f32> = vec![0.0; n]; // L_ii
        for i in 0..n {
            diag[i] = quality[i] * quality[i];
        }

        // 4. 贪心选择 (增量 Cholesky)
        let mut selected: Vec<usize> = Vec::with_capacity(k);
        let mut c = vec![vec![0.0f32; n]; k]; // Cholesky 增量列
        let mut d = diag.clone(); // 当前条件方差

        for j in 0..k {
            // 找边际增益最大的 (排除已选)
            let mut best = 0;
            let mut best_val = f32::NEG_INFINITY;
            for i in 0..n {
                if selected.contains(&i) { continue; }
                if d[i] > best_val {
                    best_val = d[i];
                    best = i;
                }
            }
            selected.push(best);

            if j == k - 1 { break; }
            if d[best] < 1e-10 { break; }

            // 计算 L[best, :] 行 (按需计算，不存储完整矩阵)
            let fp_best = fingerprints[best];
            let q_best = quality[best];

            for i in 0..n {
                // S_ij: SimHash 语义区相似度
                let fp_i = fingerprints[i];
                let semantic_best = fp_best & SimHash::MASK_SEMANTIC;
                let semantic_i = fp_i & SimHash::MASK_SEMANTIC;
                let hamming = (semantic_best ^ semantic_i).count_ones() as f32;
                let sim = 1.0 - hamming / 32.0;

                let l_val = q_best * sim * quality[i];

                // Cholesky 更新
                let mut c_j_i = l_val;
                for p in 0..j {
                    c_j_i -= c[p][best] * c[p][i];
                }
                c[j][i] = c_j_i / d[best].sqrt();
            }

            // 更新条件方差
            for i in 0..n {
                d[i] -= c[j][i] * c[j][i];
                if d[i] < 0.0 { d[i] = 0.0; }
            }
        }

        selected
    }

    /// 模拟 LLM 维护过程：对话后分析关键词关联并更新 Ontology
    /// V2: 逻辑仲裁触发器 (Logical Arbitration Trigger)
    /// 当 action 为 "replace" 时调用此函数
    /// 返回值：需要发送给 LLM2 (仲裁者) 的 Context (局部子图文本)
    pub fn trigger_arbitration(&self, source: &str) -> Option<String> {
        let src_id = self.keyword_to_node.get(&source.to_lowercase())?;
        
        // 提取 1-hop 子图
        // 格式: "Source -> Target (Strength: 0.x)"
        let mut context_lines = Vec::new();
        if let Some(edges) = self.ontology_graph.get(src_id) {
            for edge in edges {
                if let Some(target_node) = self.nodes.get(&edge.target_node_id) {
                    let strength = edge.connection_strength as f32 / 65535.0;
                    context_lines.push(format!("{} -> {} (Strength: {:.2})", 
                        source, target_node.content, strength));
                }
            }
        }
        
        if context_lines.is_empty() {
            return None;
        }
        
        Some(context_lines.join("\n"))
    }

    /// V2: 执行仲裁结果 (Apply Arbitration)
    /// 根据 LLM2 的指示删除指定关联
    pub fn apply_arbitration(&mut self, source: &str, delete_targets: Vec<String>) {
        if let Some(&src_id) = self.keyword_to_node.get(&source.to_lowercase()) {
            if let Some(edges) = self.ontology_graph.get_mut(&src_id) {
                let initial_len = edges.len();
                
                // 过滤掉需要删除的目标
                // 注意：这里需要通过 target content 反查 id，或者遍历 edges 检查 content
                // 为了性能，我们先收集要删除的 target_ids
                let mut target_ids_to_remove = Vec::new();
                
                for target_str in delete_targets {
                    if let Some(&tgt_id) = self.keyword_to_node.get(&target_str.to_lowercase()) {
                        target_ids_to_remove.push(tgt_id);
                    }
                }
                
                if !target_ids_to_remove.is_empty() {
                    edges.retain(|e| !target_ids_to_remove.contains(&e.target_node_id));
                    let removed_count = initial_len - edges.len();
                    if removed_count > 0 {
                        println!("✂️ [Arbitration] 已从 '{}' 移除 {} 条过时关联", source, removed_count);
                    }
                }
            }
        }
    }

    /// 统一维护接口 (Unified Maintenance Interface)
    /// 自动处理 upsert/replace 逻辑
    /// 返回值: Option<String> - 如果需要仲裁 (Replace 模式)，返回 1-hop 局部子图上下文；否则返回 None
    pub fn execute_maintenance(&mut self, action: &str, source: &str, target: &str, relation_type: &str, strength: f32, _reason: &str) -> Option<String> {
        match action.to_lowercase().as_str() {
            "upsert" => {
                // Upsert: 直接维护本体关联
                self.maintain_ontology(source, target, relation_type, strength);
                None
            },
            "replace" => {
                // Replace: 先应用新变更，然后触发仲裁
                // 这样 LLM2 能看到冲突的全貌 (旧 + 新)
                self.maintain_ontology(source, target, relation_type, strength);
                self.trigger_arbitration(source)
            },
            _ => {
                println!("⚠️ 未知操作: {} (Source: {})", action, source);
                None
            }
        }
    }
}

fn run_ten_million_test(count: usize) {
    println!("🔥 开始执行千万级压力测试 (目标: {} 节点) 🔥", count);
    let mut engine = AdvancedEngine::new();
    
    // 自动加载模型以支持混沌检索测试
    let model = embedding::CandleModel::new().ok();
    if let Some(m) = model {
        println!("🧠 已自动加载 {}维 Candle 向量模型用于压力测试", m.dimension);
        engine.embedding_model = Some(m);
    }
    
    // 加载数据
    engine.load_million_test_data(count);
    
    // 编译
    let start_compile = Instant::now();
    engine.compile();
    println!("⚙️ 引擎编译耗时: {:?}", start_compile.elapsed());
    
    // 1. 纯理性检索测试 (chaos_level = 0.0)
    let query = "这是一个关于 feat_42 和 feat_999 的模拟查询";
    println!("\n🔍 [1/2] 执行纯理性检索 (chaos_level = 0.0): \"{}\"", query);
    let start_retrieve_r = Instant::now();
    let results_r = engine.retrieve(query, 0, 0.0);
    println!("⏱️ 检索耗时: {:?}", start_retrieve_r.elapsed());
    println!("📊 召回结果数量: {}", results_r.len());
    
    // 2. 双轨检索测试 (chaos_level = 0.5)
    println!("\n🔍 [2/2] 执行双轨融合检索 (chaos_level = 0.5): \"{}\"", query);
    let start_retrieve_h = Instant::now();
    let results_h = engine.retrieve(query, 0, 0.5);
    println!("⏱️ 检索耗时: {:?}", start_retrieve_h.elapsed());
    println!("📊 召回结果数量: {}", results_h.len());
    
    if let Some((id, score)) = results_h.first() {
        if let Some(node) = engine.nodes.get(id) {
            println!("🔝 最高分结果: ID={}, Score={:.4}", id, score);
            println!("📝 内容摘要: {}", node.content);
        }
    }
    
    println!("\n✅ 千万级压力测试完成。");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    // V3 基准测试模式
    if args.contains(&"--v3".to_string()) || args.contains(&"--100m".to_string()) {
        let node_count = if args.contains(&"--100m".to_string()) { 
            100_000_000 
        } else if args.contains(&"--10m".to_string()) {
            10_000_000
        } else if args.contains(&"--small".to_string()) {
            1_000
        } else { 
            1_000_000 
        };
        println!("🚀 启动 PEDSA V3 架构验证 (索引-载体分离) - 规模: {} 节点", node_count);
        
        let index_path = "pedsa_v3.idx";
        let data_path = "pedsa_v3.dat";

        // 在此作用域全局加载模型
        let model = if !args.contains(&"--no-chaos".to_string()) {
            embedding::CandleModel::new().ok()
        } else {
            None
        };

        if let Some(m) = &model {
            println!("🧠 已加载 {}维 Candle 向量模型 (BGE-Small-ZH)", m.dimension);
        } else {
            println!("🚫 混沌轨道 (Chaos Track) 已禁用 (No Vector Model Loaded)");
        }

        // --- 阶段 3: 为加权向量化准备 AC 自动机 ---
        println!("🔧 [Phase 3] 初始化加权向量化组件 (AC Automaton)...");
        let mut keywords = Vec::new();
        // 加载本体关键词
        for edge in get_ontology_data() {
            keywords.push(edge.src.to_string());
            keywords.push(edge.tgt.to_string());
        }
        // 添加测试查询的演示关键词
        keywords.push("热插入".to_string());
        keywords.push("混合扫描".to_string());
        keywords.push("Chaos".to_string());
        keywords.push("SIMD".to_string());
        
        // 去重
        keywords.sort();
        keywords.dedup();
        
        let ac_matcher = AhoCorasickBuilder::new()
            .match_kind(MatchKind::LeftmostLongest)
            .build(&keywords)
            .ok();
        
        if ac_matcher.is_some() {
            println!("✅ AC 自动机构建完成，包含 {} 个关键词", keywords.len());
        }

        // 1. 检查或生成数据
        if !std::path::Path::new(index_path).exists() {
            let start_gen = Instant::now();
            
            // 定义向量化闭包
            let vectorizer = |text: &str| -> Vec<f16> {
                if let Some(m) = &model {
                    if let Some(v) = m.vectorize(text) {
                         return v.into_iter().map(f16::from_f32).collect();
                    }
                }
                vec![f16::from_f32(0.01); 512]
            };

            if let Err(e) = generate_binary_dataset(node_count, index_path, data_path, vectorizer) {
                eprintln!("❌ 生成失败: {}", e);
                return;
            }
            println!("💾 数据生成耗时: {:?}", start_gen.elapsed());
        }

        // 2. 极速加载 (mmap)
        println!("\n📥 正在加载 V3 存储引擎 (mmap)...");
        let start_load = Instant::now();
        let mut storage = match StorageEngine::new(index_path, data_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("❌ 加载失败: {}", e);
                return;
            }
        };
        println!("⚡ V3 加载完成! 耗时: {:?} (包含 Header 解析)", start_load.elapsed());

        // 测试热插入
            println!("📥 正在测试热插入功能...");
            let hot_node_text = "这是通过热插入添加的新节点，用于验证 LSM-tree 混合扫描。";
            let hot_node_fp = SimHash::compute_multimodal(hot_node_text, 0, 0, 0);
            
            // Chaos 向量生成
            let chaos_vec = if let Some(m) = &model {
                 if let Some(v) = m.vectorize(hot_node_text) {
                     v.into_iter().map(f16::from_f32).collect()
                 } else {
                     vec![f16::from_f32(0.0); 512]
                 }
            } else {
                 vec![f16::from_f32(0.0); 512]
            };
            let chaos_fp = StorageEngine::quantize_vector(&chaos_vec);
            
            if let Err(e) = storage.insert_node(999999999, hot_node_fp, hot_node_text.to_string(), 1, chaos_fp, &chaos_vec) {
                eprintln!("❌ 热插入失败: {}", e);
            } else {
                println!("✅ 已成功热插入新节点 (ID: 999999999)");
            }

        println!("📚 节点总数: {} (磁盘: {} + 缓冲区: 1)", storage.node_count(), node_count);

        // 3. 模拟共振检索 (SIMD 加速)
    let query = "验证热插入的混合扫描";
    let query_fp = SimHash::compute_multimodal(query, 0, 0, 0);
    
    println!("\n🔍 开始执行 {} 节点全量混合扫描 (SIMD + Buffer)...", storage.node_count());
    let start_scan = Instant::now();
    
    let (idx, score) = storage.scan_simd(query_fp);

    let duration = start_scan.elapsed();
    println!("⏱️ SimHash 扫描耗时: {:?}", duration);
    
    println!("🔝 Top-1 Index: {}, Score: {:.4}", idx, score);
    println!("🆔 Node ID: {}", storage.get_id(idx));
    // 只有在这里才去触碰冷数据
    println!("📝 懒加载文本: {}", storage.get_node_text_by_idx(idx));

    // --- Chaos 向量扫描 ---
    println!("\n🧠 执行 Chaos Vector 语义检索 (Top-5)...");
    let start_vec = Instant::now();
    let query_vec = if let Some(m) = &model {
         // 阶段 3: 加权向量化
         let weighted_ranges = Vec::new();
         // ... (AC 匹配逻辑已省略，因为不再在 vectorize_weighted 中使用)

         if let Some(v) = m.vectorize_weighted(query, &weighted_ranges) {
            v.into_iter().map(f16::from_f32).collect()
        } else {
            vec![f16::from_f32(0.0); 512]
        }
    } else {
        vec![f16::from_f32(0.0); 512]
    };

    let vec_results = storage.scan_vector_top_k(&query_vec, 5);
    let vec_duration = start_vec.elapsed();
    println!("⏱️ Vector 扫描耗时: {:?}", vec_duration);
    
    for (rank, (v_idx, v_score)) in vec_results.iter().enumerate() {
        let node_id = storage.get_id(*v_idx);
        
        // 检索 Chaos 指纹和向量以演示访问
        let fingerprint = storage.get_chaos_fingerprint_by_idx(*v_idx);
        let vector = storage.get_chaos_vector_by_idx(*v_idx);
        
        let text = storage.get_node_text_by_idx(*v_idx);
        println!("   #{}: ID={}, Score={:.4}, FP={:032x}, VecLen={}, Text={}", 
                 rank+1, node_id, v_score, fingerprint, vector.len(), text);
    }

    // --- 混合扫描 (Chaos FP u128 -> Chaos Vec f16) ---
    println!("\n⚡ 执行 Hybrid Scan (L1 Chaos FP u128 -> L2 Chaos Vector)...");
    let query_chaos_fp = StorageEngine::quantize_vector(&query_vec);
    println!("   Query Chaos FP: {:032x}", query_chaos_fp);

    let start_hybrid = Instant::now();
    // L1: 通过汉明距离检索前 1000 个候选者
    // L2: 通过余弦相似度重新排序前 1000 个
    let hybrid_results = storage.search_hybrid(query_chaos_fp, &query_vec, 5, 1000); 
    let hybrid_duration = start_hybrid.elapsed();
    println!("⏱️ Hybrid 扫描耗时: {:?}", hybrid_duration);

    for (rank, (v_idx, v_score)) in hybrid_results.iter().enumerate() {
        let node_id = storage.get_id(*v_idx);
        let text = storage.get_node_text_by_idx(*v_idx);
        println!("   #{}: ID={}, Score={:.4}, Text={}", 
                 rank+1, node_id, v_score, text);
    }

    // --- 模拟 V2 双层检索架构 ---
    println!("\n🎭 模拟 V2 双层检索架构 (Ontology vs Event):");
    
    // Step 1: 扫描 Ontology (node_type = 0)
    let start_ont = Instant::now();
    let (ont_idx, ont_score) = storage.scan_simd_filtered(query_fp, Some(0));
    println!("   🧠 Ontology 最佳匹配: ID={}, Score={:.4} (耗时: {:?})", 
             storage.get_id(ont_idx), ont_score, start_ont.elapsed());
    
    // Step 2: 扫描 Event (node_type = 1)
    let start_evt = Instant::now();
    let (evt_idx, evt_score) = storage.scan_simd_filtered(query_fp, Some(1));
    println!("   📅 Event 最佳匹配:    ID={}, Score={:.4} (耗时: {:?})", 
             storage.get_id(evt_idx), evt_score, start_evt.elapsed());

    // 打印内存占用提示
    println!("\n💡 提示: 请检查任务管理器中的内存占用。");
        let expected_mem = (node_count as f64 * 32.0) / (1024.0 * 1024.0);
        println!("   预期: 显存/物理内存仅占用约 {:.2}MB (32 bytes * {} nodes)", expected_mem, node_count);
        return;
    }

    if args.contains(&"--million".to_string()) || args.contains(&"--10m".to_string()) {
        run_ten_million_test(10_000_000);
        return;
    }

    println!("=== PEDSA RAG-less 高级实验框架 ===");
    let mut engine = AdvancedEngine::new();

    // 0. 加载模型 (如果存在)
    if let Ok(model) = embedding::CandleModel::new() {
        println!("🧠 已加载 {}维 Candle 向量模型 (BGE-M3 GGUF)", model.dimension);
        engine.embedding_model = Some(model);
    }

    // 1. 加载数据与建立时序脊梁
    engine.load_standard_data();

    // 2. 编译引擎
    engine.compile();

    // 3. 执行测试场景
    tests::run_all_scenarios(&mut engine);
}
