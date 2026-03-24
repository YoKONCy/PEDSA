use ahash::AHashMap;
use rayon::prelude::*;

use crate::core::simhash::SimHash;
use crate::core::types::*;
use crate::core::engine::AdvancedEngine;

// ============================================================================
// 多级检索管线 + DPP 多样性采样
// ============================================================================

impl AdvancedEngine {
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
        #[cfg(feature = "gliner")]
        let query_fp = if let Some(gliner) = &self.gliner_engine {
            let (type_entities, time_entities) = gliner.extract_all(&query_lower);
            
            // TYPE: 取最高置信度的类型
            let type_val = crate::ml::gliner_ner::best_type_val(&type_entities);
            
            // TEMPORAL: 取最高置信度且可解析的时间 span
            let timestamp = crate::ml::gliner_ner::best_timestamp(&time_entities, ref_time);
            
            // AFFECTIVE: 保持现有关键词匹配 (GLiNER 不适合情感分类)
            let emotion = SimHash::extract_emotion(&query_lower);
            
            SimHash::compute_multimodal(&query_lower, timestamp, emotion, type_val)
        } else {
            SimHash::compute_for_query(&query_lower, ref_time)
        };
        #[cfg(not(feature = "gliner"))]
        let query_fp = SimHash::compute_for_query(&query_lower, ref_time);

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
        if (query_fp & SimHash::MASK_TEMPORAL) != 0 {
            let st_hash = ((query_fp & SimHash::MASK_TEMPORAL) >> 32) as u16;
            if let Some(candidates) = self.temporal_index.get(&st_hash) {
                for &id in candidates {
                    let entry = activated_keywords.entry(id).or_insert(0.0);
                    if *entry < 0.6 { *entry = 0.6; }
                }
            }
        }

        // --- Step 1.6: 情感共振 (Affective Resonance) ---
        if (query_fp & SimHash::MASK_AFFECTIVE) != 0 {
            let emotion_hash = ((query_fp & SimHash::MASK_AFFECTIVE) >> 48) as u8;
            for i in 0..8 {
                if (emotion_hash & (1 << i)) != 0 {
                    if let Some(candidates) = self.affective_index.get(&(1 << i)) {
                         for &id in candidates {
                            let entry = activated_keywords.entry(id).or_insert(0.0);
                            if *entry < 0.7 { *entry = 0.7; }
                        }
                    }
                }
            }
        }

        // --- Step 1.9: BM25 暴力兜底 (Cold-Start Fallback) ---
        if activated_keywords.is_empty() {
            #[cfg(feature = "gliner")]
            let query_tokens: Vec<String> = if let Some(gliner) = &self.gliner_engine {
                gliner.jieba.cut(&query_lower, false)
                    .into_iter()
                    .filter(|t| t.len() >= 2)
                    .map(|t| t.to_string())
                    .collect()
            } else {
                let chars: Vec<char> = query_lower.chars().collect();
                chars.windows(2).map(|w| w.iter().collect::<String>()).collect()
            };
            #[cfg(not(feature = "gliner"))]
            let query_tokens: Vec<String> = {
                let chars: Vec<char> = query_lower.chars().collect();
                chars.windows(2).map(|w| w.iter().collect::<String>()).collect()
            };

            if !query_tokens.is_empty() {
                let k1: f32 = 1.2;
                let b: f32 = 0.75;
                
                let event_nodes: Vec<&Node> = self.nodes.values()
                    .filter(|n| n.node_type == NodeType::Event)
                    .collect();
                let n_docs = event_nodes.len() as f32;
                if n_docs > 0.0 {
                    let avg_dl: f32 = event_nodes.iter()
                        .map(|n| n.content.len() as f32)
                        .sum::<f32>() / n_docs;

                    let mut df: AHashMap<&str, u32> = AHashMap::new();
                    for token in &query_tokens {
                        for node in &event_nodes {
                            if node.content.contains(token.as_str()) {
                                *df.entry(token.as_str()).or_default() += 1;
                            }
                        }
                    }

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
                    
                    if !bm25_scores.is_empty() {
                        bm25_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                        let max_score = bm25_scores[0].1;
                        
                        for (id, score) in bm25_scores.iter().take(20) {
                            let normalized = score / max_score * 0.4;
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
                    
                    let degree = self.in_degrees.get(&edge.target_node_id).unwrap_or(&1);
                    let inhibition_factor = 1.0 / (1.0 + (*degree as f32).log10()); 
                    
                    if edge.edge_type == SimHash::EDGE_EQUALITY {
                        let entry = ontology_expanded.entry(edge.target_node_id).or_insert(0.0);
                        if score > *entry {
                             *entry = score;
                        }
                        continue;
                    }
                    
                    let log_weight = (1.0 + weight * 65535.0).ln() / (65536.0_f32).ln();
                    let energy = score * log_weight * 0.95 * inhibition_factor;
                    
                    if edge.edge_type == SimHash::EDGE_INHIBITION {
                        let entry = ontology_expanded.entry(edge.target_node_id).or_insert(0.0);
                        *entry -= energy; 
                    } else {
                        if energy < 0.05 { continue; }
                        
                        let entry = ontology_expanded.entry(edge.target_node_id).or_insert(0.0);
                        *entry = (*entry).max(energy);
                    }
                }
            }
        }

        // --- Step 3: 能量归一化 (Energy Normalization) ---
        let total_energy: f32 = ontology_expanded.values().sum();
        if total_energy > 10.0 {
            let factor = 10.0 / total_energy;
            for val in ontology_expanded.values_mut() {
                *val *= factor;
            }
        }

        // --- Step 4: 第二数据库 (Memory 记忆库) 扩散 ---
        let final_scores = ontology_expanded.clone();
        let decay = 0.85;
        let layer_limit = 5000; 

        let mut seeds: Vec<(&i64, &f32)> = ontology_expanded.iter().collect();
        seeds.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
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
                            
                            let degree = self.in_degrees.get(&edge.target_node_id).unwrap_or(&1);
                            let inhibition_factor = 1.0 / (1.0 + (*degree as f32).log10());

                            let log_weight = (1.0 + weight * 65535.0).ln() / (65536.0_f32).ln();
                            let energy = score * log_weight * decay * inhibition_factor;
                            
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
                let semantic_sim = SimHash::similarity_weighted(query_fp, node.fingerprint, SimHash::MASK_SEMANTIC);
                let mut resonance_boost = semantic_sim * 0.6;
                
                // 2. 时间共振
                if (query_fp & SimHash::MASK_TEMPORAL) != 0 {
                    let temporal_sim = SimHash::similarity_weighted(query_fp, node.fingerprint, SimHash::MASK_TEMPORAL);
                    resonance_boost += temporal_sim * 0.5;
                }

                // 3. 情感共鸣
                if (query_fp & SimHash::MASK_AFFECTIVE) != 0 {
                    let query_emotions = (query_fp & SimHash::MASK_AFFECTIVE) >> 48;
                    let node_emotions = (node.fingerprint & SimHash::MASK_AFFECTIVE) >> 48;
                    
                    if (query_emotions & node_emotions) != 0 {
                        resonance_boost += 0.6; 
                    }
                }

                // 4. 类型对齐
                if (query_fp & SimHash::MASK_TYPE) != 0 {
                    let type_sim = SimHash::similarity_weighted(query_fp, node.fingerprint, SimHash::MASK_TYPE);
                    resonance_boost += type_sim * 0.8;
                }

                // 5. 艾宾浩斯记忆衰减 (Ebbinghaus Decay)
                let current_decay_time = if ref_time > 0 { ref_time } else { 1777593600 }; 
                let tau = 31536000.0;
                
                if node.timestamp > 0 && node.timestamp < current_decay_time {
                    let delta_t = (current_decay_time - node.timestamp) as f32;
                    let decay_factor = (-delta_t / tau).exp();
                    
                    let final_decay = decay_factor.max(0.8);
                    *score *= final_decay;
                }

                *score += resonance_boost;
            }
        }

        // --- 第四阶段：混沌激活 (双轨并行) ---
        if chaos_level > 0.0 {
            if let Some((query_chaos_fp, query_vec_f16)) = self.calculate_chaos(query) {
                let mut combined_results = AHashMap::new();
                
                for (id, score) in results.iter() {
                    combined_results.insert(*id, *score * (1.0 - chaos_level));
                }

                // --- 1. L1 粗排 (1-bit 量化) ---
                let mut candidates: Vec<(usize, u32)> = Vec::with_capacity(self.chaos_store.ids.len() / 10);

                for (idx, &node_fp) in self.chaos_store.fingerprints.iter().enumerate() {
                    let distance = query_chaos_fp.hamming_distance(&node_fp);
                    
                    if distance < 256 {
                        candidates.push((idx, distance));
                    }
                }

                candidates.sort_unstable_by_key(|k| k.1);
                
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
                            
                            if sim > 0.6 {
                                let normalized = (sim - 0.6) / 0.4;
                                let chaos_score = normalized * 0.15;
                                let weighted_chaos = chaos_score * chaos_level;
                                
                                *combined_results.entry(node_id).or_default() += weighted_chaos;
                            }
                        }
                    }
                }
                
                let mut final_results: Vec<_> = combined_results.into_iter().collect();
                final_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                return final_results;
            }
        }
        
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // --- Step 6: DPP 贪心多样性采样 (Determinantal Point Process) ---
        if results.len() > 10 {
            let dpp_candidates = results.len().min(50);
            let dpp_select = 10;
            let selected_indices = self.dpp_greedy_select(&results[..dpp_candidates], dpp_select);
            
            let mut dpp_results: Vec<(i64, f32)> = selected_indices.iter()
                .map(|&i| results[i])
                .collect();
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

        // 1. 构建质量因子 q_i = score^0.8
        let quality: Vec<f32> = candidates.iter()
            .map(|(_, s)| s.max(1e-10).powf(0.8))
            .collect();

        // 2. 构建相似度矩阵 S_ij (基于 SimHash 汉明距离)
        let fingerprints: Vec<u64> = candidates.iter()
            .map(|(id, _)| self.nodes.get(id).map_or(0, |n| n.fingerprint))
            .collect();

        // 3. L-ensemble 核矩阵
        let mut diag: Vec<f32> = vec![0.0; n];
        for i in 0..n {
            diag[i] = quality[i] * quality[i];
        }

        // 4. 贪心选择 (增量 Cholesky)
        let mut selected: Vec<usize> = Vec::with_capacity(k);
        let mut c = vec![vec![0.0f32; n]; k];
        let mut d = diag.clone();

        for j in 0..k {
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

            let fp_best = fingerprints[best];
            let q_best = quality[best];

            for i in 0..n {
                let fp_i = fingerprints[i];
                let semantic_best = fp_best & SimHash::MASK_SEMANTIC;
                let semantic_i = fp_i & SimHash::MASK_SEMANTIC;
                let hamming = (semantic_best ^ semantic_i).count_ones() as f32;
                let sim = 1.0 - hamming / 32.0;

                let l_val = q_best * sim * quality[i];

                let mut c_j_i = l_val;
                for p in 0..j {
                    c_j_i -= c[p][best] * c[p][i];
                }
                c[j][i] = c_j_i / d[best].sqrt();
            }

            for i in 0..n {
                d[i] -= c[j][i] * c[j][i];
                if d[i] < 0.0 { d[i] = 0.0; }
            }
        }

        selected
    }
}
