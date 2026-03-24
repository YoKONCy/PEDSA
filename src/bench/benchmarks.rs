use aho_corasick::{AhoCorasickBuilder, MatchKind};
use half::f16;
use std::time::Instant;

use crate::core::simhash::SimHash;
use crate::data::storage::{StorageEngine, generate_binary_dataset};
use crate::ml::embedding;
use crate::core::engine::AdvancedEngine;
use crate::data::dataset::get_ontology_data;

pub fn run_ten_million_test(count: usize) {
    println!("🔥 开始执行千万级压力测试 (目标: {} 节点) 🔥", count);
    let mut engine = AdvancedEngine::new();

    let model = embedding::CandleModel::new().ok();
    if let Some(m) = model {
        println!("🧠 已自动加载 {}维 Candle 向量模型用于压力测试", m.dimension);
        engine.embedding_model = Some(m);
    }

    engine.load_million_test_data(count);

    let start_compile = Instant::now();
    engine.compile();
    println!("⚙️ 引擎编译耗时: {:?}", start_compile.elapsed());

    let query = "这是一个关于 feat_42 和 feat_999 的模拟查询";
    println!("\n🔍 [1/2] 执行纯理性检索 (chaos_level = 0.0): \"{}\"", query);
    let start_retrieve_r = Instant::now();
    let results_r = engine.retrieve(query, 0, 0.0);
    println!("⏱️ 检索耗时: {:?}", start_retrieve_r.elapsed());
    println!("📊 召回结果数量: {}", results_r.len());

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

pub fn run_v3_benchmark(args: &[String]) {
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

    println!("🔧 [Phase 3] 初始化加权向量化组件 (AC Automaton)...");
    let mut keywords = Vec::new();
    for edge in get_ontology_data() {
        keywords.push(edge.src.to_string());
        keywords.push(edge.tgt.to_string());
    }
    keywords.push("热插入".to_string());
    keywords.push("混合扫描".to_string());
    keywords.push("Chaos".to_string());
    keywords.push("SIMD".to_string());
    keywords.sort();
    keywords.dedup();

    let ac_matcher = AhoCorasickBuilder::new()
        .match_kind(MatchKind::LeftmostLongest)
        .build(&keywords)
        .ok();

    if ac_matcher.is_some() {
        println!("✅ AC 自动机构建完成，包含 {} 个关键词", keywords.len());
    }

    if !std::path::Path::new(index_path).exists() {
        let start_gen = Instant::now();
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

    println!("\n📥 正在加载 V3 存储引擎 (mmap)...");
    let start_load = Instant::now();
    let mut storage = match StorageEngine::new(index_path, data_path) {
        Ok(s) => s,
        Err(e) => { eprintln!("❌ 加载失败: {}", e); return; }
    };
    println!("⚡ V3 加载完成! 耗时: {:?} (包含 Header 解析)", start_load.elapsed());

    // 热插入测试
    run_hot_insert_test(&mut storage, &model);

    println!("📚 节点总数: {} (磁盘: {} + 缓冲区: 1)", storage.node_count(), node_count);

    // SimHash 扫描
    run_simhash_scan(&storage, &model);

    // 双层检索
    run_dual_layer_test(&storage);

    let expected_mem = (node_count as f64 * 32.0) / (1024.0 * 1024.0);
    println!("\n💡 提示: 请检查任务管理器中的内存占用。");
    println!("   预期: 显存/物理内存仅占用约 {:.2}MB (32 bytes * {} nodes)", expected_mem, node_count);
}

fn run_hot_insert_test(storage: &mut StorageEngine, model: &Option<embedding::CandleModel>) {
    println!("📥 正在测试热插入功能...");
    let hot_node_text = "这是通过热插入添加的新节点，用于验证 LSM-tree 混合扫描。";
    let hot_node_fp = SimHash::compute_multimodal(hot_node_text, 0, 0, 0);
    let chaos_vec = if let Some(m) = model {
        if let Some(v) = m.vectorize(hot_node_text) {
            v.into_iter().map(f16::from_f32).collect()
        } else { vec![f16::from_f32(0.0); 512] }
    } else { vec![f16::from_f32(0.0); 512] };
    let chaos_fp = StorageEngine::quantize_vector(&chaos_vec);

    if let Err(e) = storage.insert_node(999999999, hot_node_fp, hot_node_text.to_string(), 1, chaos_fp, &chaos_vec) {
        eprintln!("❌ 热插入失败: {}", e);
    } else {
        println!("✅ 已成功热插入新节点 (ID: 999999999)");
    }
}

fn run_simhash_scan(storage: &StorageEngine, model: &Option<embedding::CandleModel>) {
    let query = "验证热插入的混合扫描";
    let query_fp = SimHash::compute_multimodal(query, 0, 0, 0);

    println!("\n🔍 开始执行 {} 节点全量混合扫描 (SIMD + Buffer)...", storage.node_count());
    let start_scan = Instant::now();
    let (idx, score) = storage.scan_simd(query_fp);
    println!("⏱️ SimHash 扫描耗时: {:?}", start_scan.elapsed());
    println!("🔝 Top-1 Index: {}, Score: {:.4}", idx, score);
    println!("🆔 Node ID: {}", storage.get_id(idx));
    println!("📝 懒加载文本: {}", storage.get_node_text_by_idx(idx));

    println!("\n🧠 执行 Chaos Vector 语义检索 (Top-5)...");
    let start_vec = Instant::now();
    let query_vec = if let Some(m) = model {
        let weighted_ranges = Vec::new();
        if let Some(v) = m.vectorize_weighted(query, &weighted_ranges) {
            v.into_iter().map(f16::from_f32).collect()
        } else { vec![f16::from_f32(0.0); 512] }
    } else { vec![f16::from_f32(0.0); 512] };

    let vec_results = storage.scan_vector_top_k(&query_vec, 5);
    println!("⏱️ Vector 扫描耗时: {:?}", start_vec.elapsed());
    for (rank, (v_idx, v_score)) in vec_results.iter().enumerate() {
        let node_id = storage.get_id(*v_idx);
        let fingerprint = storage.get_chaos_fingerprint_by_idx(*v_idx);
        let vector = storage.get_chaos_vector_by_idx(*v_idx);
        let text = storage.get_node_text_by_idx(*v_idx);
        println!("   #{}: ID={}, Score={:.4}, FP={:032x}, VecLen={}, Text={}",
                 rank+1, node_id, v_score, fingerprint, vector.len(), text);
    }

    println!("\n⚡ 执行 Hybrid Scan (L1 Chaos FP u128 -> L2 Chaos Vector)...");
    let query_chaos_fp = StorageEngine::quantize_vector(&query_vec);
    println!("   Query Chaos FP: {:032x}", query_chaos_fp);
    let start_hybrid = Instant::now();
    let hybrid_results = storage.search_hybrid(query_chaos_fp, &query_vec, 5, 1000);
    println!("⏱️ Hybrid 扫描耗时: {:?}", start_hybrid.elapsed());
    for (rank, (v_idx, v_score)) in hybrid_results.iter().enumerate() {
        let node_id = storage.get_id(*v_idx);
        let text = storage.get_node_text_by_idx(*v_idx);
        println!("   #{}: ID={}, Score={:.4}, Text={}", rank+1, node_id, v_score, text);
    }
}

fn run_dual_layer_test(storage: &StorageEngine) {
    let query = "验证热插入的混合扫描";
    let query_fp = SimHash::compute_multimodal(query, 0, 0, 0);

    println!("\n🎭 模拟 V2 双层检索架构 (Ontology vs Event):");
    let start_ont = Instant::now();
    let (ont_idx, ont_score) = storage.scan_simd_filtered(query_fp, Some(0));
    println!("   🧠 Ontology 最佳匹配: ID={}, Score={:.4} (耗时: {:?})",
             storage.get_id(ont_idx), ont_score, start_ont.elapsed());

    let start_evt = Instant::now();
    let (evt_idx, evt_score) = storage.scan_simd_filtered(query_fp, Some(1));
    println!("   📅 Event 最佳匹配:    ID={}, Score={:.4} (耗时: {:?})",
             storage.get_id(evt_idx), evt_score, start_evt.elapsed());
}
