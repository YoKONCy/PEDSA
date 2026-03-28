use pyo3::prelude::*;
use crate::core::engine::AdvancedEngine;

#[pyclass(name = "Engine")]
pub struct PedsaEngine { inner: AdvancedEngine }

#[pymethods]
impl PedsaEngine {
    #[new]
    fn new() -> Self { Self { inner: AdvancedEngine::new() } }
    
    fn load_embedding_model(&mut self) -> PyResult<bool> {
        match crate::ml::embedding::CandleModel::new() {
            Ok(model) => { self.inner.embedding_model = Some(model); Ok(true) }
            Err(_) => Ok(false)
        }
    }
    
    fn load_gliner_model(&mut self, model_dir: &str) -> PyResult<bool> {
        #[cfg(feature = "gliner")]
        {
            if let Ok(e) = crate::ml::gliner_ner::GlinerEngine::new(model_dir) {
                self.inner.gliner_engine = Some(e); Ok(true)
            } else { Ok(false) }
        }
        #[cfg(not(feature = "gliner"))]
        { let _ = model_dir; Ok(false) }
    }
    
    fn add_feature(&mut self, id: i64, keyword: &str) { self.inner.add_feature(id, keyword); }
    #[pyo3(signature = (id, summary, timestamp=0, emotion=0, event_type=0))]
    fn add_event(&mut self, id: i64, summary: &str, timestamp: u64, emotion: u8, event_type: u8) { self.inner.add_event(id, summary, timestamp, emotion, event_type); }
    fn add_edge(&mut self, src: i64, tgt: i64, weight: f32) { self.inner.add_edge(src, tgt, weight); }
    fn maintain_ontology(&mut self, src: &str, tgt: &str, rel: &str, s: f32) { self.inner.maintain_ontology(src, tgt, rel, s); }
    fn compile(&mut self) { self.inner.compile(); self.inner.build_temporal_backbone(); }
    #[pyo3(signature = (query, ref_time=0, chaos_level=0.0))]
    fn retrieve(&self, query: &str, ref_time: u64, chaos_level: f32) -> Vec<(i64, f32)> { self.inner.retrieve(query, ref_time, chaos_level) }
    
    fn node_count(&self) -> usize { self.inner.tdb.node_count() }
    fn feature_count(&self) -> usize { self.inner.keyword_to_node.len() }
    fn get_or_create_feature(&mut self, word: &str) -> i64 { self.inner.get_or_create_feature(word) }
    
    #[pyo3(signature = (action, source, target, relation_type, strength, reason=""))]
    fn execute_maintenance(&mut self, action: &str, source: &str, target: &str, relation_type: &str, strength: f32, reason: &str) -> Option<String> {
        self.inner.execute_maintenance(action, source, target, relation_type, strength, reason)
    }

    fn apply_arbitration(&mut self, source: &str, delete_targets: Vec<String>) { self.inner.apply_arbitration(source, delete_targets); }
    fn trigger_arbitration(&self, source: &str) -> Option<String> { self.inner.trigger_arbitration(source) }

    fn get_node(&self, id: i64) -> Option<PyObject> {
        Python::with_gil(|py| {
            let payload = self.inner.tdb.get_payload(id as u64)?;
            let dict = pyo3::types::PyDict::new(py);
            dict.set_item("id", id).ok()?;
            dict.set_item("content", payload.get("content").and_then(|v| v.as_str()).unwrap_or("")).ok()?;
            dict.set_item("fingerprint", payload.get("fingerprint").and_then(|v| v.as_u64()).unwrap_or(0)).ok()?;
            dict.set_item("timestamp", payload.get("timestamp").and_then(|v| v.as_u64()).unwrap_or(0)).ok()?;
            dict.set_item("prev_event", payload.get("prev_event").and_then(|v| v.as_i64())).ok()?;
            dict.set_item("next_event", payload.get("next_event").and_then(|v| v.as_i64())).ok()?;
            dict.set_item("type", payload.get("type").and_then(|v| v.as_str()).unwrap_or("")).ok()?;
            dict.set_item("emotions", payload.get("emotions").and_then(|v| v.as_u64()).unwrap_or(0)).ok()?;
            Some(dict.into())
        })
    }
    
    fn get_edges(&self, node_id: i64) -> Vec<(i64, f32, String)> {
        self.inner.tdb.get_edges(node_id as u64).into_iter()
            .map(|e| (e.target_id as i64, e.weight, e.label.clone())).collect()
    }
    fn get_ontology_edges(&self, node_id: i64) -> Vec<(i64, f32, String)> { self.get_edges(node_id) }
    fn all_node_ids(&self) -> Vec<i64> { self.inner.tdb.all_node_ids().into_iter().map(|id| id as i64).collect() }
    fn all_feature_keywords(&self) -> Vec<String> { self.inner.keyword_to_node.keys().cloned().collect() }
    fn keyword_to_id(&self, keyword: &str) -> Option<i64> { self.inner.keyword_to_node.get(&keyword.to_lowercase()).copied() }
    fn apply_decay(&mut self, decay_rate: f32, threshold: u16) -> usize { self.inner.apply_global_decay_and_pruning(decay_rate, threshold) }

    fn load_standard_data(&mut self) { self.inner.load_standard_data(); }
    fn export_to_sqlite(&self, path: &str) { println!("V2 Architecture uses native TriviumDB persistence via Mmap. No SQLite export needed."); }
    fn import_from_sqlite(&self, path: &str) { println!("V2 Architecture uses native TriviumDB persistence via Mmap. No SQLite import needed."); }
}

#[pymodule]
fn pedsa(m: &Bound<'_, PyModule>) -> PyResult<()> { m.add_class::<PedsaEngine>()?; Ok(()) }
