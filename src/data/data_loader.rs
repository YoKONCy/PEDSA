#![allow(dead_code)]
#![allow(unused_imports)]
use std::hash::{Hash, Hasher};
use std::time::Instant;
use twox_hash::XxHash64;

use crate::core::engine::AdvancedEngine;
use crate::data::dataset::{get_tech_domain_data, get_social_domain_data, get_history_domain_data,
                     get_value_domain_data, get_daily_domain_data, get_timeline_domain_data,
                     get_ontology_data};

impl AdvancedEngine {
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

        for ev in all_events {
            self.add_event(ev.id, ev.summary, 0, 0, 0);
            for feature in ev.features {
                let feature_lower = feature.to_lowercase();
                let mut s = XxHash64::with_seed(0);
                feature_lower.hash(&mut s);
                let feat_id = (s.finish() as i64).abs();
                self.add_feature(feat_id, &feature_lower);
                self.add_edge(feat_id, ev.id, 1.0);
            }
        }

        println!("📚 正在注入定义库 (Ontology) 数据...");
        let ontology_edges = get_ontology_data();
        for edge in ontology_edges {
            let relation_type = if edge.is_equality {
                "equality"
            } else if edge.is_inhibition {
                "inhibition"
            } else {
                "representation"
            };
            self.maintain_ontology(edge.src, edge.tgt, relation_type, edge.weight);
        }

        for edge in all_edges {
            self.add_edge(edge.src, edge.tgt, edge.weight);
        }

        self.add_edge(205, 100, 0.6);
        self.add_edge(200, 302, 0.4);
        self.build_temporal_backbone();
    }

    pub fn load_million_test_data(&mut self, node_count: usize) {
        println!("🏗️ 正在生成 {} 级大规模合成数据...", node_count);
        let start = Instant::now();
        
        
        
        

        let feature_count = 50_000;
        for i in 0..feature_count {
            let id = i as i64 + 1_000_000_000;
            let kw = format!("feat_{}", i);
            self.add_feature(id, &kw);
        }

        let event_count = node_count;
        for i in 0..event_count {
            let id = i as i64 + 2_000_000_000;
            let summary = format!("这是一个模拟的事件总结节点，编号为 {}，用于进行规模压力测试。PEDSA 算法应当在这种规模下依然保持极高的检索效率。", i);
            self.add_event(id, &summary, 0, 0, 0);
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
}
