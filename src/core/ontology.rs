use std::hash::{Hash, Hasher};
use twox_hash::XxHash64;
use crate::core::engine::AdvancedEngine;
use crate::core::stopwords;

impl AdvancedEngine {
    pub fn maintain_ontology(&mut self, source: &str, target: &str, relation_type: &str, strength: f32) {
        println!("🤖 [LLM Maintenance] 发现新关联: {} -> {} (type: {}, strength: {})", source, target, relation_type, strength);
        let src_id = self.get_or_create_feature(source);
        let tgt_id = self.get_or_create_feature(target);
        let edge_type = match relation_type.to_lowercase().as_str() {
            "equality" | "equal" => "equality",
            "inhibition" | "conflict" => "inhibition",
            _ => "representation",
        };
        // Use Triviumdb exact graph relations. TriviumDB supports PPR/teleporting 
        // without edge weights having to be purely u16!
        let _ = self.tdb.link(src_id as u64, tgt_id as u64, edge_type, strength);
        if edge_type == "equality" || edge_type == "inhibition" {
            let _ = self.tdb.link(tgt_id as u64, src_id as u64, edge_type, strength);
        }
    }

    #[allow(dead_code)]
    pub fn apply_global_decay_and_pruning(&mut self, _decay_rate: f32, _threshold: u16) -> usize {
        // Since we delegated topology to TriviumDB, TriviumDB should handle edge decay.
        // For now, we skip manual edge decay from high level Rust.
        println!("[PEDSA Memory] Simulated Pruning executed (Delegated to TriviumDB Compaction).");
        0
    }

    pub fn get_or_create_feature(&mut self, word: &str) -> i64 {
        let word_lower = word.to_lowercase();
        if stopwords::is_stopword(&word_lower) { return -1; }
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

    #[allow(dead_code)]
    pub fn trigger_arbitration(&self, source: &str) -> Option<String> {
        let src_id = self.keyword_to_node.get(&source.to_lowercase())?;
        let mut context_lines = Vec::new();
        // Since nodes is gone, we fetch via get_edges
        for edge in self.tdb.get_edges(*src_id as u64) {
            if let Some(payload) = self.tdb.get_payload(edge.target_id) {
                if let Some(content) = payload.get("content").and_then(|v| v.as_str()) {
                    context_lines.push(format!("{} -> {} (Strength: {:.2})", source, content, edge.weight));
                }
            }
        }
        if context_lines.is_empty() { return None; }
        Some(context_lines.join("
"))
    }

    #[allow(dead_code)]
    pub fn apply_arbitration(&mut self, source: &str, delete_targets: Vec<String>) {
        if let Some(&src_id) = self.keyword_to_node.get(&source.to_lowercase()) {
            for target_str in delete_targets {
                if let Some(&tgt_id) = self.keyword_to_node.get(&target_str.to_lowercase()) {
                    let _ = self.tdb.unlink(src_id as u64, tgt_id as u64);
                }
            }
            println!("✂️ [Arbitration] 已从 '{}' 移除了过时关联", source);
        }
    }

    #[allow(dead_code)]
    pub fn execute_maintenance(&mut self, action: &str, source: &str, target: &str, relation_type: &str, strength: f32, _reason: &str) -> Option<String> {
        match action.to_lowercase().as_str() {
            "upsert" => { self.maintain_ontology(source, target, relation_type, strength); None },
            "replace" => { self.maintain_ontology(source, target, relation_type, strength); self.trigger_arbitration(source) },
            _ => None
        }
    }
}
