use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};
use ahash::AHashMap;
use smallvec::SmallVec;
use half::f16;


use crate::core::simhash::SimHash;
use crate::core::types::*;
use crate::data::storage::{StorageEngine, ChaosFingerprint};
use crate::ml::embedding::CandleModel;
#[cfg(feature = "gliner")]
use crate::ml::gliner_ner::GlinerEngine;
use crate::core::stopwords;

// ============================================================================
// 高级实验引擎
// ============================================================================

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

    // V2: 异步接口 (预留，待接入 PyO3 后启用)
    #[allow(dead_code)]
    pub async_task: Box<dyn AsyncTaskInterface + Send + Sync>,

    // 第四阶段: Candle 嵌入模型
    pub embedding_model: Option<CandleModel>,

    // GLiNER-X-Base: 实体类型 + 时间 span 提取 (替代硬编码 contains())
    #[cfg(feature = "gliner")]
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
            #[cfg(feature = "gliner")]
            gliner_engine: None,
        }
    }

    /// 添加特征节点
    pub fn add_feature(&mut self, id: i64, keyword: &str) {
        let keyword_lower = keyword.to_lowercase();
        
        // --- 停用词硬过滤 (双保险机制) ---
        if stopwords::is_stopword(&keyword_lower) {
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
    pub fn extract_timestamp(text: &str) -> u64 {
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
        #[cfg(feature = "gliner")]
        let type_val = if let Some(gliner) = &self.gliner_engine {
            let (type_entities, time_entities) = gliner.extract_all(summary);
            
            // 如果原始解析器未提取到时间，用 GLiNER 补充
            if timestamp == 0 && !time_entities.is_empty() {
                let ref_time = instant::Instant::now().elapsed().as_secs(); // 近似
                timestamp = crate::ml::gliner_ner::best_timestamp(&time_entities, ref_time);
            }
            
            crate::ml::gliner_ner::best_type_val(&type_entities)
        } else {
            SimHash::TYPE_UNKNOWN
        };
        #[cfg(not(feature = "gliner"))]
        let type_val = SimHash::TYPE_UNKNOWN;

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

    /// 建立双向时序链表 (Temporal Backbone)
    pub fn build_temporal_backbone(&mut self) {
        println!("⏳ 正在构建时序脊梁 (Temporal Backbone)...");
        
        // 1. 收集所有 Event 节点并按时间戳排序
        let mut events: Vec<(i64, u64)> = self.nodes.values()
            .filter(|n| n.node_type == NodeType::Event)
            .map(|n| (n.id, n.timestamp))
            .collect();
        
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
        #[cfg(feature = "gliner")]
        {
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
        }

        println!("🚀 引擎编译完成：{} 个特征锚点, {} 个总节点, {} 个时空桶, {} 个情感维度", 
            self.feature_keywords.len(), self.nodes.len(), self.temporal_index.len(), self.affective_index.len());
    }
}
