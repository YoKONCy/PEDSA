use std::hash::{Hash, Hasher};
use twox_hash::XxHash64;

use crate::core::types::*;
use crate::core::engine::AdvancedEngine;
use crate::core::stopwords;

// ============================================================================
// 本体维护 + LTD 衰减剪枝 + 逻辑仲裁
// ============================================================================

impl AdvancedEngine {
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
            "equality" | "equal" => 1, // SimHash::EDGE_EQUALITY
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
        // 1. Equality (Type 1): 强制双向
        // 2. Inhibition (Type 255): 强制双向
        // 3. Representation (Type 0): 默认单向
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

    pub fn get_or_create_feature(&mut self, word: &str) -> i64 {
        let word_lower = word.to_lowercase();
        
        // 停用词检查
        if stopwords::is_stopword(&word_lower) {
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

    /// 模拟 LLM 维护过程：对话后分析关键词关联并更新 Ontology
    /// V2: 逻辑仲裁触发器 (Logical Arbitration Trigger)
    /// 返回值：需要发送给 LLM2 (仲裁者) 的 Context (局部子图文本)
    #[allow(dead_code)]
    pub fn trigger_arbitration(&self, source: &str) -> Option<String> {
        let src_id = self.keyword_to_node.get(&source.to_lowercase())?;
        
        // 提取 1-hop 子图
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
    #[allow(dead_code)]
    pub fn apply_arbitration(&mut self, source: &str, delete_targets: Vec<String>) {
        if let Some(&src_id) = self.keyword_to_node.get(&source.to_lowercase()) {
            if let Some(edges) = self.ontology_graph.get_mut(&src_id) {
                let initial_len = edges.len();
                
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
    #[allow(dead_code)]
    pub fn execute_maintenance(&mut self, action: &str, source: &str, target: &str, relation_type: &str, strength: f32, _reason: &str) -> Option<String> {
        match action.to_lowercase().as_str() {
            "upsert" => {
                self.maintain_ontology(source, target, relation_type, strength);
                None
            },
            "replace" => {
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
