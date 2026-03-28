#![allow(dead_code)]
#![allow(unused_imports)]
use aho_corasick::{AhoCorasickBuilder, MatchKind};
use half::f16;
use std::time::Instant;

use crate::core::simhash::SimHash;
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
        if let Some(node) = engine.tdb.get_payload(*id as u64) {
            println!("🔝 最高分结果: ID={}, Score={:.4}", id, score);
            println!("📝 内容摘要: {}", node.get("content").unwrap().as_str().unwrap());
        }
    }
    println!("\n✅ 千万级压力测试完成。");
}

pub fn run_v2_benchmark(args: &[String]) {
    let node_count = if args.contains(&"--100m".to_string()) {
        100_000_000
    } else if args.contains(&"--10m".to_string()) {
        10_000_000
    } else if args.contains(&"--small".to_string()) {
        1_000
    } else {
        1_000_000
    };
    println!("🚀 启动 PEDSA V2 架构验证 (接入 TriviumDB) - 数量: {} ", node_count);
    let mut engine = AdvancedEngine::new();
    println!("✅ PEDSA 已经与 TriviumDB 完成整合。由于采用纯内存+mmap架构，测试将通过引擎原生运行。");
    // 复用千万级压力测试方法来跑压测
    engine.load_million_test_data(node_count);
}
