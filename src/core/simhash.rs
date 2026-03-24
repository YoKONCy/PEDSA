use std::hash::{Hash, Hasher};
use twox_hash::XxHash64;

// ============================================================================
// 语义指纹 (SimHash V2: Partitioned Multimodal)
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
    #[allow(dead_code)]
    pub const TYPE_LOCATION: u8 = 0x04;  // 地点
    pub const TYPE_OBJECT: u8 = 0x05;    // 物件
    #[allow(dead_code)]
    pub const TYPE_VALUES: u8 = 0x06;    // 价值观

    // --- 边类型常量 (V3.5 类型化边 - 简化版) ---
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn similarity(a: u64, b: u64) -> f32 {
        // 默认全区匹配
        Self::similarity_weighted(a, b, 0xFFFFFFFFFFFFFFFF)
    }
}
