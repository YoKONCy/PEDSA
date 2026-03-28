use triviumdb::database::SearchConfig;
use crate::core::simhash::SimHash;
use crate::core::engine::AdvancedEngine;

impl AdvancedEngine {
    pub fn retrieve(&self, query: &str, ref_time: u64, chaos_level: f32) -> Vec<(i64, f32)> {
        let query_lower = query.to_lowercase();

        #[cfg(feature = "gliner")]
        let query_fp = if let Some(gliner) = &self.gliner_engine {
            let (type_e, time_e) = gliner.extract_all(&query_lower);
            let type_val = crate::ml::gliner_ner::best_type_val(&type_e);
            let timestamp = crate::ml::gliner_ner::best_timestamp(&time_e, ref_time);
            let emotion = SimHash::extract_emotion(&query_lower);
            SimHash::compute_multimodal(&query_lower, timestamp, emotion, type_val)
        } else { SimHash::compute_for_query(&query_lower, ref_time) };
        
        #[cfg(not(feature = "gliner"))]
        let query_fp = SimHash::compute_for_query(&query_lower, ref_time);

        let query_vec_f32 = self.calculate_chaos(query);
        let config = SearchConfig {
            top_k: 200, 
            expand_depth: 2,
            min_score: 0.1,
            teleport_alpha: 0.15,
            enable_advanced_pipeline: true,
            enable_bq_coarse_search: chaos_level > 0.0,
            text_boost: 1.5,
            enable_text_hybrid_search: true, // Native fast search
            enable_inverse_inhibition: true, // Native inverse inhibition
            lateral_inhibition_threshold: 5000, 
            enable_dpp: false, // Disabling native DPP so we can rerank via SimHash first
            ..Default::default()
        };

        let mut hits = self.tdb.search_hybrid(
            Some(query), 
            query_vec_f32.as_deref(), 
            &config
        ).unwrap_or_default();

        let current_decay_time = if ref_time > 0 { ref_time } else { 1777593600 }; 
        let tau = 31536000.0;

        // V2 Temporal Decay & Multimodal Resonance
        for hit in &mut hits {
            if let Some(timestamp) = hit.payload.get("timestamp").and_then(|v| v.as_u64()) {
                if timestamp > 0 && timestamp < current_decay_time {
                    let delta_t = (current_decay_time - timestamp) as f32;
                    let decay_factor = (-delta_t / tau).exp();
                    hit.score *= decay_factor.max(0.8);
                }
            }
            if let Some(fp) = hit.payload.get("fingerprint").and_then(|v| v.as_u64()) {
                let semantic_sim = SimHash::similarity_weighted(query_fp, fp, SimHash::MASK_SEMANTIC);
                let mut boost = semantic_sim * 0.6;
                if (query_fp & SimHash::MASK_TEMPORAL) != 0 { boost += SimHash::similarity_weighted(query_fp, fp, SimHash::MASK_TEMPORAL) * 0.5; }
                if (query_fp & SimHash::MASK_AFFECTIVE) != 0 {
                    if ((query_fp & SimHash::MASK_AFFECTIVE) >> 48) & ((fp & SimHash::MASK_AFFECTIVE) >> 48) != 0 { boost += 0.6; }
                }
                if (query_fp & SimHash::MASK_TYPE) != 0 { boost += SimHash::similarity_weighted(query_fp, fp, SimHash::MASK_TYPE) * 0.8; }
                
                hit.score += boost;
            }
        }
        
        hits.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // Native PEDSA DPP Greedy
        let results: Vec<(i64, f32)> = hits.iter().map(|h| (h.id as i64, h.score)).collect();
        if results.len() > 10 {
            let dpp_candidates = results.len().min(50);
            let selected = self.dpp_greedy_select(&hits[..dpp_candidates], 10);
            let mut dpp_results: Vec<(i64, f32)> = selected.iter().map(|&i| results[i]).collect();
            for item in results.iter().skip(dpp_candidates) { dpp_results.push(*item); }
            return dpp_results;
        }

        results
    }

    fn dpp_greedy_select(&self, candidates: &[triviumdb::node::SearchHit], k: usize) -> Vec<usize> {
        let n = candidates.len();
        if n <= k { return (0..n).collect(); }
        let quality: Vec<f32> = candidates.iter().map(|s| s.score.max(1e-10).powf(0.8)).collect();
        let fingerprints: Vec<u64> = candidates.iter().map(|h| h.payload.get("fingerprint").and_then(|v| v.as_u64()).unwrap_or(0)).collect();
        let mut diag: Vec<f32> = quality.iter().map(|q| q * q).collect();
        
        let mut selected = Vec::with_capacity(k);
        let mut c = vec![vec![0.0f32; n]; k];

        for j in 0..k {
            let mut best = 0; let mut best_val = f32::NEG_INFINITY;
            for i in 0..n {
                if !selected.contains(&i) && diag[i] > best_val { best_val = diag[i]; best = i; }
            }
            selected.push(best);
            if j == k - 1 || diag[best] < 1e-10 { break; }

            let fp_best = fingerprints[best];
            let q_best = quality[best];

            for i in 0..n {
                let sim = 1.0 - ((fp_best & SimHash::MASK_SEMANTIC) ^ (fingerprints[i] & SimHash::MASK_SEMANTIC)).count_ones() as f32 / 32.0;
                let mut c_j_i = q_best * sim * quality[i];
                for p in 0..j { c_j_i -= c[p][best] * c[p][i]; }
                c[j][i] = c_j_i / diag[best].sqrt();
                diag[i] = (diag[i] - c[j][i] * c[j][i]).max(0.0);
            }
        }
        selected
    }
}
