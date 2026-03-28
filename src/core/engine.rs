use ahash::AHashMap;
use triviumdb::Database;
use serde_json::json;

use crate::core::simhash::SimHash;
use crate::ml::embedding::CandleModel;
#[cfg(feature = "gliner")]
use crate::ml::gliner_ner::GlinerEngine;
use crate::core::stopwords;

pub struct AdvancedEngine {
    pub tdb: Database<f32>,
    pub keyword_to_node: AHashMap<String, i64>,
    pub embedding_model: Option<CandleModel>,
    #[cfg(feature = "gliner")]
    pub gliner_engine: Option<GlinerEngine>,
}

impl AdvancedEngine {
    pub fn new() -> Self {
        let db = Database::open(".trivium_pedsa", 512).unwrap();
        let mut keyword_to_node = AHashMap::new();
        
        for id in db.all_node_ids() {
            if let Some(payload) = db.get_payload(id) {
                if payload.get("type").and_then(|v| v.as_str()) == Some("feature") {
                    if let Some(content) = payload.get("content").and_then(|v| v.as_str()) {
                        keyword_to_node.insert(content.to_lowercase(), id as i64);
                    }
                }
            }
        }
        
        Self {
            tdb: db,
            keyword_to_node,
            embedding_model: None,
            #[cfg(feature = "gliner")]
            gliner_engine: None,
        }
    }

    pub fn extract_timestamp(text: &str) -> u64 {
        let default_ts = 1672531200;
        for (year_idx, _) in text.match_indices("年") {
            if year_idx >= 4 && text.is_char_boundary(year_idx - 4) {
                if let Ok(year) = text[year_idx-4..year_idx].parse::<i32>() {
                    let mut day = 1;
                    let rest = &text[year_idx+3..];
                    if let Some(month_idx) = rest.find("月") {
                        if month_idx <= 5 {
                            let m_str = rest[..month_idx].trim();
                            if let Ok(month) = m_str.parse::<i32>() {
                                let rest_day = &rest[month_idx+3..];
                                if let Some(day_idx) = rest_day.find("日") {
                                    if day_idx <= 5 {
                                        let d_str = rest_day[..day_idx].trim();
                                        if let Ok(d) = d_str.parse::<i32>() {
                                            day = d;
                                        }
                                    }
                                }
                                return (year as u64 - 1970) * 31536000 + (month as u64) * 2592000 + (day as u64) * 86400;
                            }
                        }
                    }
                }
            }
        }
        default_ts
    }

    pub fn calculate_chaos(&self, text: &str) -> Option<Vec<f32>> {
        let model = self.embedding_model.as_ref()?;
        let weighted_ranges = Vec::new();
        model.vectorize_weighted(text, &weighted_ranges)
    }

    pub fn add_feature(&mut self, id: i64, keyword: &str) {
        let keyword_lower = keyword.to_lowercase();
        if stopwords::is_stopword(&keyword_lower) { return; }

        let _ = self.tdb.insert_with_id(
            id as u64,
            &vec![0.0; 512],
            json!({
                "type": "feature",
                "content": keyword_lower,
                "fingerprint": SimHash::compute(&keyword_lower),
                "timestamp": 0
            })
        );
        self.tdb.index_keyword(id as u64, &keyword_lower).ok();
        self.keyword_to_node.insert(keyword_lower, id);
    }

    pub fn add_event(&mut self, id: i64, summary: &str, explicit_timestamp: u64, explicit_emotion: u8, explicit_type: u8) {
        let mut timestamp = if explicit_timestamp > 0 { explicit_timestamp } else { Self::extract_timestamp(summary) };
        let emotion_val = if explicit_emotion > 0 { explicit_emotion } else { SimHash::extract_emotion(summary) };

        #[cfg(feature = "gliner")]
        let type_val = if explicit_type > 0 { 
            explicit_type 
        } else if let Some(gliner) = &self.gliner_engine {
            let (type_entities, time_entities) = gliner.extract_all(summary);
            if timestamp == 0 && !time_entities.is_empty() {
                let ref_time = 1711200000;
                timestamp = crate::ml::gliner_ner::best_timestamp(&time_entities, ref_time);
            }
            crate::ml::gliner_ner::best_type_val(&type_entities)
        } else { SimHash::TYPE_UNKNOWN };
        
        #[cfg(not(feature = "gliner"))]
        let type_val = if explicit_type > 0 { explicit_type } else { SimHash::TYPE_UNKNOWN };

        let fingerprint = SimHash::compute_multimodal(summary, timestamp, emotion_val, type_val);
        let payload = json!({
             "type": "event",
             "content": summary,
             "timestamp": timestamp,
             "fingerprint": fingerprint,
             "emotions": emotion_val
        });

        if let Some(vec) = self.calculate_chaos(summary) {
             let _ = self.tdb.insert_with_id(id as u64, &vec, payload);
        } else {
             let _ = self.tdb.insert_with_id(id as u64, &vec![0.0; 512], payload);
        }
        self.tdb.index_text(id as u64, summary).ok();
    }

    pub fn add_edge(&mut self, src: i64, tgt: i64, weight: f32) {
        let _ = self.tdb.link(src as u64, tgt as u64, "memory_edge", weight);
    }

    pub fn build_temporal_backbone(&mut self) {
        println!("⏳ 正在构建时序脊梁 (Temporal Backbone) [TriviumDB 版]...");
        let mut events: Vec<(i64, u64)> = Vec::new();
        
        for id in self.tdb.all_node_ids() {
            if let Some(payload) = self.tdb.get_payload(id) {
                if payload.get("type").and_then(|v| v.as_str()) == Some("event") {
                    let ts = payload.get("timestamp").and_then(|v| v.as_u64()).unwrap_or(0);
                    events.push((id as i64, ts));
                }
            }
        }
        
        events.sort_by(|a, b| {
            if a.1 != b.1 { a.1.cmp(&b.1) } else { a.0.cmp(&b.0) }
        });

        for i in 0..events.len() {
            let (curr_id, _) = events[i];
            let mut payload = self.tdb.get_payload(curr_id as u64).unwrap();
            
            if i > 0 { payload["prev_event"] = json!(events[i-1].0); }
            if i < events.len() - 1 { payload["next_event"] = json!(events[i+1].0); }
            
            let _ = self.tdb.update_payload(curr_id as u64, payload);
        }
        println!("✅ 时序脊梁构建完成，已串联 {} 个事件节点。", events.len());
    }

    pub fn compile(&mut self) {
        self.tdb.build_text_index().ok();
        
        #[cfg(feature = "gliner")]
        {
            if let Ok(mut engine) = GlinerEngine::new("models/gliner-x-base") {
                let mut custom_count = 0;
                for (kw, _) in &self.keyword_to_node {
                    if kw.len() >= 2 {
                        engine.add_custom_word(kw);
                        custom_count += 1;
                    }
                }
                println!("🏷️  GLiNER-X-Base 已加载 (ONNX Runtime), {} 个自定义词", custom_count);
                self.gliner_engine = Some(engine);
            }
        }
        self.tdb.flush().unwrap();
        println!("🚀 引擎编译/落盘完成：共 {} 个底层存储节点", self.tdb.node_count());
    }
}
