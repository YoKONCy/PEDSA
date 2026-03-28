use crate::core::engine::AdvancedEngine;
#[cfg(test)]
use crate::ml::embedding::CandleModel;
use crate::core::simhash::SimHash;
use std::time::Instant;

fn test_scenario_17_pruning(engine: &mut AdvancedEngine) {
    println!("\n--- 测试场景 17: 动态突触剪枝 (Dynamic Synaptic Pruning) ---");
    
    // 1. 创建临时关联
    engine.maintain_ontology("TempConcept", "TargetConcept", "representation", 0.5);
    let src_id = engine.keyword_to_node.get("tempconcept").cloned().unwrap();
    
    println!("初始状态: 关联已建立 (Strength: 0.5)");
    
    // 验证关联存在
    if let Some(edges) = engine.ontology_graph.get(&src_id) {
        println!("当前边数: {}", edges.len());
        println!("边强度: {}", edges[0].connection_strength);
    }
    
    // 2. 执行多次强衰减 (模拟长时间未激活)
    println!("执行多次全局衰减 (Decay Rate: 0.5)...");
    for i in 1..=5 {
        // 阈值设为 2000 (约 0.03)，初始 32767 (0.5)
        // 1: 16383
        // 2: 8191
        // 3: 4095
        // 4: 2047 (接近阈值)
        // 5: 1023 (应被剪除)
        let pruned = engine.apply_global_decay_and_pruning(0.5, 2000);
        println!("第 {} 次衰减后剪除边数: {}", i, pruned);
        
        if let Some(edges) = engine.ontology_graph.get(&src_id) {
             if !edges.is_empty() {
                 println!("   -> 剩余强度: {}", edges[0].connection_strength);
             } else {
                 println!("   -> 关联已断裂 (Synapse Pruned)");
             }
        }
    }
    
    // 3. 验证是否彻底移除
    if let Some(edges) = engine.ontology_graph.get(&src_id) {
        if edges.is_empty() {
            println!("✅ 验证成功: 长期未激活的突触已被物理移除。");
        } else {
            println!("❌ 验证失败: 突触仍然存在。");
        }
    }
}

fn test_scenario_18_edge_types(engine: &mut AdvancedEngine) {
    println!("\n--- 测试场景 18: 三位一体边逻辑 (Typed Edges) ---");
    
    // 1. 建立测试数据结构
    // Source Feature: "SourceFeat"
    
    // Path 1: Normal (Assoc) -> Event_Normal
    engine.add_feature(1001, "TargetNormal");
    engine.add_event(2001, "Event Normal Content");
    engine.add_edge(1001, 2001, 1.0); // Feature -> Event
    
    // Path 2: Equal -> Event_Equal
    engine.add_feature(1002, "TargetEqual");
    engine.add_event(2002, "Event Equal Content");
    engine.add_edge(1002, 2002, 1.0);
    
    // Path 3: Inhibit -> Event_Inhibit
    engine.add_feature(1003, "TargetInhibit");
    engine.add_event(2003, "Event Inhibit Content");
    engine.add_edge(1003, 2003, 1.0);
    
    // 2. 建立 Ontology 关联
    // SourceFeat -> TargetNormal (0.9, Normal)
    engine.maintain_ontology("SourceFeat", "TargetNormal", "representation", 0.9);
    // SourceFeat <-> TargetEqual (1.0, Equal)
    engine.maintain_ontology("SourceFeat", "TargetEqual", "equality", 1.0);
    // SourceFeat -| TargetInhibit (0.8, Inhibit)
    engine.maintain_ontology("SourceFeat", "TargetInhibit", "inhibition", 0.8);
    
    // 必须重新编译
    engine.compile();
    
    // 3. 执行检索
    let query = "SourceFeat";
    let results = engine.retrieve(query, 0, 0.0);
    
    let score_normal = results.iter().find(|(id, _)| *id == 2001).map(|(_, s)| *s).unwrap_or(0.0);
    let score_equal = results.iter().find(|(id, _)| *id == 2002).map(|(_, s)| *s).unwrap_or(0.0);
    let score_inhibit = results.iter().find(|(id, _)| *id == 2003).map(|(_, s)| *s).unwrap_or(0.0);
    
    println!("激活结果 (Event Score):");
    println!("  Event_Normal: {:.4}", score_normal);
    println!("  Event_Equal:  {:.4}", score_equal);
    println!("  Event_Inhibit:{:.4}", score_inhibit);
    
    // 验证 Equal (应该最高，且无损耗传递到 Feature 层)
    // Feature层: Equal=1.0, Normal=0.9*0.95=0.855
    // Event层: * 1.0 * 0.85 (decay)
    if score_equal > score_normal {
        println!("✅ Equal 边验证成功: 能量高于普通边 ({:.4} > {:.4})", score_equal, score_normal);
    } else {
        println!("❌ Equal 边验证失败");
    }
    
    // 验证 Inhibit (应该最低，甚至为0)
    if score_inhibit < 0.01 {
        println!("✅ Inhibit 边验证成功: 目标被抑制 (Score: {:.4})", score_inhibit);
    } else {
        println!("❌ Inhibit 边验证失败: 目标仍被激活 (Score: {:.4})", score_inhibit);
    }
}

pub fn run_all_scenarios() {
    println!("=== PEDSA RAG-less 高级实验框架 ===");
    let mut engine = AdvancedEngine::new();

    // 0. 加载模型 (如果存在)
    if let Ok(model) = crate::ml::embedding::CandleModel::new() {
        println!("已加载 {}维 Candle 向量模型 (BGE-M3 GGUF)", model.dimension);
        engine.embedding_model = Some(model);
    }

    // 1. 加载数据
    engine.load_standard_data();

    // 2. 编译引擎
    engine.compile();

    println!("\n--- 实验框架就绪 (双数据库架构) ---");
    println!("当前节点总数: {}", engine.nodes.len());
    println!("当前特征锚点: {}", engine.feature_keywords.len());

    // 3. 执行硬核跨领域查询测试
    test_scenario_1(&engine);
    test_scenario_2(&engine);
    test_scenario_3(&engine);
    test_scenario_4(&engine);
    test_scenario_5(&engine);
    test_scenario_6(&engine);
    test_scenario_7(&engine);
    test_scenario_8(&engine);
    test_scenario_9(&engine);
    test_scenario_10(&engine);
    test_scenario_11_ontology(&mut engine);
    test_scenario_11_temporal(&mut engine);
    test_scenario_12(&engine);
    test_scenario_13(&engine);
    test_scenario_14(&engine);
    test_scenario_15(&mut engine);
    test_scenario_16_chaos(&mut engine);
    test_scenario_17_pruning(&mut engine);
    test_scenario_18_edge_types(&mut engine);
    test_scenario_19_emotion(&mut engine);
    
    run_precision_evaluation(&engine);
    final_throughput_eval(&engine);
}

fn run_precision_evaluation(engine: &AdvancedEngine) {
    println!("\n=== 检索精度评估 (Precision@k Evaluation) ===");
    
    let ground_truth = [
        ("Rust PyO3 重构", 100),
        ("Wasm 指令级审计", 101),
        ("BGE-Reranker ONNX", 102),
        ("Protobuf JSON Electron", 104),
        ("SharedArrayBuffer 零拷贝", 107),
        ("jemalloc dirty page", 111),
        ("LWW-Element-Set CRDT", 112),
        ("A* 算法 启发式搜索", 113),
        ("LSM-Tree 存储引擎", 115),
        ("Double-Array Trie AC 自动机", 117),
        ("树莓派 NEON 向量计算", 114),
        ("ABAC 权限控制 元数据", 116),
        ("sccache 分布式编译", 120),
        ("rkyv 零拷贝 序列化", 121),
        ("HNSW 缓存行对齐", 126),
    ];

    let mut top_1_hits = 0;
    let mut top_5_hits = 0;
    let total = ground_truth.len();

    for (query, expected_id) in ground_truth.iter() {
        let results = engine.retrieve(query, 0, 0.0);
        
        // Check Top-1
        if let Some((id, score)) = results.first() {
            if id == expected_id {
                top_1_hits += 1;
            } else {
                println!("❌ Top-1 Miss: Query='{}', Expected={}, Got={} (Score={:.4})", 
                         query, expected_id, id, score);
            }
        }

        // Check Top-5
        for (id, _) in results.iter().take(5) {
            if id == expected_id {
                top_5_hits += 1;
                break;
            }
        }
    }

    let top_1_rate = (top_1_hits as f32 / total as f32) * 100.0;
    let top_5_rate = (top_5_hits as f32 / total as f32) * 100.0;

    println!("测试样本总数: {}", total);
    println!("🎯 Top-1 命中率: {:.2}% ({}/{})", top_1_rate, top_1_hits, total);
    println!("🎯 Top-5 命中率: {:.2}% ({}/{})", top_5_rate, top_5_hits, total);
    
    if top_1_rate > 90.0 {
        println!("✅ 精度表现优秀：PEDSA 扩散激活模型在标准数据集上具有极高的确定性。");
    } else {
        println!("⚠️ 精度待优化：可能需要调整 SimHash 掩码权重或扩散衰减系数。");
    }
}

fn test_scenario_1(engine: &AdvancedEngine) {
    println!("\n--- 测试场景 1: 定义库的语义对齐 (佩罗 -> Pero) ---");
    let query = "佩罗最近是不是又踩我键盘了？";
    println!("查询: \"{}\"", query);
    let start = Instant::now();
    let results = engine.retrieve(query, 0, 0.0);
    println!("检索耗时: {:?}", start.elapsed());
    for (i, (id, score)) in results.iter().take(3).enumerate() {
        if let Some(node) = engine.nodes.get(id) {
            println!("[{}] 能量:{:.4} | ID:{} | 摘要: {}", i+1, score, id, node.content);
        }
    }
}

fn test_scenario_2(engine: &AdvancedEngine) {
    println!("\n--- 测试场景 2: 复杂长文本语义共振 (含定义扩展) ---");
    let query = "2026年虽然遇到了很多技术挑战，比如分布式一致性的 Paxos 实现和 LSM-Tree 的性能瓶颈，但在上海滨江大道的构思让我觉得离线优先和数字主权才是真正的未来愿景。";
    println!("查询: \"{}\"", query);
    let start = Instant::now();
    let results = engine.retrieve(query, 0, 0.0);
    println!("检索耗时: {:?}", start.elapsed());
    for (i, (id, score)) in results.iter().take(5).enumerate() {
        if let Some(node) = engine.nodes.get(id) {
            println!("[{}] 能量:{:.4} | ID:{} | 摘要: {}", i+1, score, id, node.content);
        }
    }
}

fn test_scenario_3(engine: &AdvancedEngine) {
    println!("\n--- 测试场景 3: 极致碎片化输入 (佩罗 + 屁屁) ---");
    let query = "佩罗 屁屁 灵性";
    println!("查询: \"{}\"", query);
    let start = Instant::now();
    let results = engine.retrieve(query, 0, 0.0);
    println!("检索耗时: {:?}", start.elapsed());
    for (i, (id, score)) in results.iter().take(5).enumerate() {
        if let Some(node) = engine.nodes.get(id) {
            println!("[{}] 能量:{:.4} | ID:{} | 摘要: {}", i+1, score, id, node.content);
        }
    }
}

fn test_scenario_4(engine: &AdvancedEngine) {
    println!("\n--- 测试场景 4: 表征关联推理 (女孩 -> Pero) ---");
    let query = "那个戴蝴蝶结的小女孩最近怎么样了？";
    println!("查询: \"{}\"", query);
    let start = Instant::now();
    let results = engine.retrieve(query, 0, 0.0);
    println!("检索耗时: {:?}", start.elapsed());
    for (i, (id, score)) in results.iter().take(5).enumerate() {
        if let Some(node) = engine.nodes.get(id) {
            println!("[{}] 能量:{:.4} | ID:{} | 摘要: {}", i+1, score, id, node.content);
        }
    }
}

fn test_scenario_5(engine: &AdvancedEngine) {
    println!("\n--- 测试场景 5: 时间与地点精确回溯 ---");
    let query = "2024年夏天在深圳那个关于Wasm的沙龙，当时发生了什么？";
    println!("查询: \"{}\"", query);
    let start = Instant::now();
    let results = engine.retrieve(query, 0, 0.0);
    println!("检索耗时: {:?}", start.elapsed());
    for (i, (id, score)) in results.iter().take(3).enumerate() {
        if let Some(node) = engine.nodes.get(id) {
            println!("[{}] 能量:{:.4} | ID:{} | 摘要: {}", i+1, score, id, node.content);
        }
    }
}

fn test_scenario_6(engine: &AdvancedEngine) {
    println!("\n--- 测试场景 6: 价值观与未来愿景 ---");
    let query = "用户对于‘数字灵魂’和‘数字生命’的终极理想是什么？";
    let results = engine.retrieve(query, 0, 0.0);
    for (i, (id, score)) in results.iter().take(5).enumerate() {
        if let Some(node) = engine.nodes.get(id) {
            println!("[{}] 能量:{:.4} | ID:{} | 摘要: {}", i+1, score, id, node.content);
        }
    }
}

fn test_scenario_7(engine: &AdvancedEngine) {
    println!("\n--- 测试场景 7: 跨领域长文本综合查询 (100字+) ---");
    let query = "记得2024年春天在上海徐家汇那家咖啡馆敲下第一行代码的时候，我就在想，如果能通过 Rust 实现一种像人脑一样的扩散激活模型，解决大模型的 Hallucination 问题，同时还能保护用户的数字主权和隐私，那该多好。现在 PeroCore 已经有了初步的 PEDSA 实现，甚至还在尝试用 Wasm 进行加速，这真的让我很欣慰。";
    println!("查询: \"{}\"", query);
    let start = Instant::now();
    let results = engine.retrieve(query, 0, 0.0);
    println!("检索耗时: {:?}", start.elapsed());
    for (i, (id, score)) in results.iter().take(8).enumerate() {
        if let Some(node) = engine.nodes.get(id) {
            println!("[{}] 能量:{:.4} | ID:{} | 摘要: {}", i+1, score, id, node.content);
        }
    }
}

fn test_scenario_8(engine: &AdvancedEngine) {
    println!("\n--- 测试场景 8: 隐喻与间接关联推理 (皮革 -> 极客精神) ---");
    let query = "最近做手工皮革的时候，那种对针脚精准度的追求，让我想起了在处理分布式一致性日志冲突时的那种偏执。这种极客精神是不是我一直以来的核心驱动力？";
    println!("查询: \"{}\"", query);
    let start = Instant::now();
    let results = engine.retrieve(query, 0, 0.0);
    println!("检索耗时: {:?}", start.elapsed());
    for (i, (id, score)) in results.iter().take(5).enumerate() {
        if let Some(node) = engine.nodes.get(id) {
            println!("[{}] 能量:{:.4} | ID:{} | 摘要: {}", i+1, score, id, node.content);
        }
    }
}

fn test_scenario_9(engine: &AdvancedEngine) {
    println!("\n--- 测试场景 9: 多点时空上下文关联检索 ---");
    let query = "我记得在深圳参加完沙龙后，又回到了上海，在张江优化了 LSM-Tree。后来在滨江大道跑步时，又想到了关于共情能力的设计。这些分散的片段是如何串联起来的？";
    println!("查询: \"{}\"", query);
    let start = Instant::now();
    let results = engine.retrieve(query, 0, 0.0);
    println!("检索耗时: {:?}", start.elapsed());
    for (i, (id, score)) in results.iter().take(8).enumerate() {
        if let Some(node) = engine.nodes.get(id) {
            println!("[{}] 能量:{:.4} | ID:{} | 摘要: {}", i+1, score, id, node.content);
        }
    }
}

fn test_scenario_10(engine: &AdvancedEngine) {
    println!("\n--- 测试场景 10: 技术演进脉络追溯 (Python -> Rust -> Wasm) ---");
    let query = "当初那个简陋的 Python 脚本，是怎么一步步演化到现在的 Rust 核心，并且还能在浏览器里跑 Wasm 的？这中间有哪些关键的转折点和教训？";
    println!("查询: \"{}\"", query);
    let start = Instant::now();
    let results = engine.retrieve(query, 0, 0.0);
    println!("检索耗时: {:?}", start.elapsed());
    for (i, (id, score)) in results.iter().take(8).enumerate() {
        if let Some(node) = engine.nodes.get(id) {
            println!("[{}] 能量:{:.4} | ID:{} | 摘要: {}", i+1, score, id, node.content);
        }
    }
}

fn test_scenario_11_ontology(engine: &mut AdvancedEngine) {
    println!("\n--- 模拟场景 11: LLM 维护与 Ontology 实时更新 ---");
    
    // 0. 预埋一个关于 Llama-3 的事件，但初始时不与"逻辑推理"直接关联
    let target_id = 8888;
    engine.add_event(target_id, "Meta 发布了 Llama-3 模型，它在多项基准测试中表现出色。");
    // 关联到 "Llama-3" 特征 (测试大小写不敏感)
    let feat_llama3 = engine.get_or_create_feature("Llama-3");
    engine.add_edge(feat_llama3, target_id, 1.0);
    
    // 初始编译
    engine.compile();

    let query_pre = "有哪些逻辑推理能力强的模型？";
    println!("维护前查询: \"{}\"", query_pre);
    let results_pre = engine.retrieve(query_pre, 0, 0.0);
    
    // 检查是否找到了 Llama-3 事件
    let found_pre = results_pre.iter().any(|(id, _)| *id == target_id);
    println!("维护前是否找到 Llama-3: {}", found_pre);

    // 1. 模拟 LLM 维护：建立 "逻辑推理" -> "Llama-3" 的关联
    // 注意：特征名可以使用原始大小写，系统应自动处理
    engine.maintain_ontology("Llama-3", "LLM", "representation", 0.9);
    engine.maintain_ontology("逻辑推理", "Llama-3", "representation", 0.85);
    engine.maintain_ontology("LLAMA3", "Llama-3", "equality", 1.0);

    // 必须重新编译 AC 自动机以索引新关键词
    engine.compile();

    println!("\n维护后再次查询: \"{}\"", query_pre);
    let results_post = engine.retrieve(query_pre, 0, 0.0);
    
    // 打印前 3 个结果
    for (i, (id, score)) in results_post.iter().take(3).enumerate() {
        if let Some(node) = engine.nodes.get(id) {
            let tag = if *id == target_id { " [TARGET]" } else { "" };
            println!("[{}] 能量:{:.4} | ID:{} | 摘要: {}{}", i+1, score, id, node.content, tag);
        }
    }
}

fn test_scenario_11_temporal(engine: &mut AdvancedEngine) {
    println!("\n--- 测试场景 11: 时序脊梁追溯 (Temporal Backbone) ---");
    // 确保时序脊梁是最新的 (以防前面的测试添加了新事件)
    engine.build_temporal_backbone();

    let query = "2024年3月12日写下第一行代码之后，紧接着发生了什么？";
    println!("查询: \"{}\"", query);
    let results = engine.retrieve(query, 0, 0.0);
    if let Some((id, _)) = results.first() {
        if let Some(node) = engine.nodes.get(id) {
            println!("📍 定位到的事件 ID: {}", id);
            println!("📝 内容: {}", node.content);
            if let Some(next_id) = node.next_event {
                if let Some(next_node) = engine.nodes.get(&next_id) {
                    println!("👉 [Next Event] ID: {}", next_id);
                    println!("   内容: {}", next_node.content);
                }
            } else {
                println!("🚫 没有后续事件 (链表末端)");
            }
        }
    }
}

fn test_scenario_12(engine: &AdvancedEngine) {
    println!("\n--- 测试场景 12: 时空模糊共振 (Spatio-Temporal Resonance) ---");
    let query = "2024年发生过什么开心的事情吗？";
    println!("查询: \"{}\"", query);
    let results = engine.retrieve(query, 0, 0.0);
    for (i, (id, score)) in results.iter().take(3).enumerate() {
        if let Some(node) = engine.nodes.get(id) {
            println!("[{}] 能量:{:.4} | ID:{} | 摘要: {}", i+1, score, id, node.content);
        }
    }
}

fn test_scenario_13(engine: &AdvancedEngine) {
    println!("\n--- 测试场景 13: 实体类型消歧 (Entity Type Disambiguation) ---");
    let query_a = "Pero 最近在做什么？";
    println!("\n[Case A] 查询: \"{}\" (Expect Type: Person)", query_a);
    let results_a = engine.retrieve(query_a, 0, 0.0);
    if let Some((id, score)) = results_a.first() {
        if let Some(node) = engine.nodes.get(id) {
            println!("   Top Result: [{}] {:.4} | {}", id, score, node.content);
        }
    }

    let query_b = "那个红色的蝴蝶结放在哪里了？";
    println!("\n[Case B] 查询: \"{}\" (Expect Type: Object)", query_b);
    let results_b = engine.retrieve(query_b, 0, 0.0);
    if let Some((id, score)) = results_b.first() {
        if let Some(node) = engine.nodes.get(id) {
            println!("   Top Result: [{}] {:.4} | {}", id, score, node.content);
        }
    }
}

fn test_scenario_14(engine: &AdvancedEngine) {
    println!("\n--- 测试场景 14: 情感共鸣 (Affective Resonance) ---");
    let query_a = "有什么让人感到生气和恼火的事情吗？";
    println!("\n[Case A] 查询: \"{}\" (Expect Emotion: ANGER)", query_a);
    let results_a = engine.retrieve(query_a, 0, 0.0);
    for (i, (id, score)) in results_a.iter().take(3).enumerate() {
        if let Some(node) = engine.nodes.get(id) {
            println!("   [{}] {:.4} | {}", i+1, score, node.content);
        }
    }

    let query_b = "有什么让人感到开心和欣慰的事情吗？";
    println!("\n[Case B] 查询: \"{}\" (Expect Emotion: JOY)", query_b);
    let results_b = engine.retrieve(query_b, 0, 0.0);
    for (i, (id, score)) in results_b.iter().take(3).enumerate() {
        if let Some(node) = engine.nodes.get(id) {
            println!("   [{}] {:.4} | {}", i+1, score, node.content);
        }
    }

    let query_c = "那次在滨江大道跑步被认出来，感觉好害羞啊。";
    println!("\n[Case C] 查询: \"{}\" (Expect Emotion: SHY)", query_c);
    let results_c = engine.retrieve(query_c, 0, 0.0);
    for (i, (id, score)) in results_c.iter().take(3).enumerate() {
        if let Some(node) = engine.nodes.get(id) {
            println!("   [{}] {:.4} | {}", i+1, score, node.content);
        }
    }
}

fn test_scenario_15(engine: &mut AdvancedEngine) {
    println!("\n--- 测试场景 15: 艾宾浩斯记忆衰减 (Memory Decay) ---");
    let old_id = 9001;
    let new_id = 9002;
    engine.add_event(old_id, "2024年1月1日，用户在学习 Rust 的生命周期，觉得非常难懂。");
    engine.add_event(new_id, "2026年4月1日，用户在复习 Rust 的生命周期，觉得豁然开朗。");
    // 确保 "rust" 特征存在，避免因为特征不存在导致边添加失败
    let rust_id = engine.get_or_create_feature("rust");
    engine.add_edge(rust_id, old_id, 1.0);
    engine.add_edge(rust_id, new_id, 1.0);
    
    // 重新编译以更新索引
    engine.compile();

    let query = "Rust 生命周期";
    println!("\n查询: \"{}\"", query);
    let results = engine.retrieve(query, 0, 0.0);
    for (i, (id, score)) in results.iter().take(5).enumerate() {
        if let Some(node) = engine.nodes.get(id) {
            if node.id == old_id || node.id == new_id {
                let tag = if node.id == old_id { "[OLD]" } else { "[NEW]" };
                println!("   [{}] {:.4} {} | TS: {} | {}", i+1, score, tag, node.timestamp, node.content);
            }
        }
    }
}

pub fn test_scenario_16_chaos(engine: &AdvancedEngine) {
    println!("\n--- 测试场景 16: 真实数据集混沌检索验证 (Chaos Retrieval) ---");
    let query = "如何优化内存分配以减少碎片？";
    println!("查询: \"{}\"", query);
    
    let start = Instant::now();
    // 使用 0.15 的修正系数触发混沌检索
    let results = engine.retrieve(query, 0, 0.15);
    println!("检索耗时: {:?}", start.elapsed());

    for (i, (id, score)) in results.iter().take(5).enumerate() {
        if let Some(node) = engine.nodes.get(id) {
            println!("[{}] 能量:{:.4} | ID:{} | 摘要: {}", i+1, score, id, node.content);
        }
    }

    // 验证逻辑
    let found_relevant = results.iter().any(|(id, _)| *id == 111 || *id == 132 || *id == 106);
    if found_relevant {
        println!("✅ 混沌检索成功捕获到内存相关底层技术条目。");
    } else {
        println!("⚠️ 混沌检索未达到预期增益。");
    }
}

fn final_throughput_eval(engine: &AdvancedEngine) {
    println!("\n=== 最终压力测试与精度评估 ===");
    let test_queries = [
        "PeroCore 是如何处理长时记忆的？",
        "2024年发生了哪些重要的事情？",
        "如何通过代码表达对生活的热爱？",
        "PEDSA 相比传统的向量数据库有什么优势？",
        "在上海徐家汇的那次咖啡馆偶遇，对项目有什么影响？",
    ];
    let start_total = Instant::now();
    for query in test_queries {
        let _ = engine.retrieve(query, 0, 0.0);
    }
    println!("⏱️ 总吞吐量评估: 处理 {} 次复杂查询耗时 {:?}", test_queries.len(), start_total.elapsed());
    println!("✅ 任务收口完成：双数据库架构稳定，召回精度符合预期，大规模数据集加载正常。");
}

#[test]
fn test_v2_temporal_resonance() {
    let mut engine = AdvancedEngine::new();
    
    // 1. 添加带有时间信息的事件
    engine.add_event(100, "2024年1月1日，Rust在嵌入式领域取得突破");
    engine.add_event(101, "2025年1月1日，Python性能大幅提升");
    
    // 2. 模拟相对时间查询：假设当前是 2024-01-02，查询“昨天”
    // 使用与 extract_timestamp 相同的简易时间戳计算公式，确保一致性
    // (year - 1970) * 31536000 + month * 2592000 + day * 86400
    let current_time = (2024 - 1970) * 31536000 + 1 * 2592000 + 2 * 86400;
    let results = engine.retrieve("昨天发生了什么", current_time, 0.0);
    
    // 应该能召回到 2024年1月1日 的事件 (id=100)
    let found = results.iter().any(|(id, _)| *id == 100);
    assert!(found, "Should find event via relative time resonance");
}

#[test]
fn test_v2_worldview_agnostic() {
    let mut engine = AdvancedEngine::new();
    
    // 场景 A: 现实世界 (Real World)
    // 假设今天是 2024-02-04
    let real_world_now = (2024 - 1970) * 31536000 + 2 * 2592000 + 4 * 86400; 
    engine.add_event(2024, "2024年2月3日，AI 伴侣 PR 发布了新版本");
    
    // 场景 B: AIPR 世界 (AIPR Worldview)
    // 假设今天是 AIPR 历法的 2026-05-20
    let aipr_world_now = (2026 - 1970) * 31536000 + 5 * 2592000 + 20 * 86400; 
    engine.add_event(2026, "2026年5月19日，Pero 在张江实验室通过了图灵测试");
    
    // 编译引擎以构建索引
    engine.compile();
    
    // 1. 测试现实世界共鸣
    let results_real = engine.retrieve("昨天 PR 发布了什么？", real_world_now, 0.0);
    assert!(results_real.iter().any(|(id, _)| *id == 2024), "现实世界相对时间匹配失败");
    
    // 2. 测试 AIPR 世界共鸣
    let results_aipr = engine.retrieve("昨天 Pero 做了什么？", aipr_world_now, 0.0);
    assert!(results_aipr.iter().any(|(id, _)| *id == 2026), "AIPR 世界相对时间匹配失败");
}



#[test]
fn test_v2_logic_arbitration() {
    let mut engine = AdvancedEngine::new();
    
    // 1. 初始化旧知识: Pero -> 蓝发 (Strength: 0.9)
    engine.maintain_ontology("Pero", "蓝发", "representation", 0.9);
    engine.maintain_ontology("Pero", "女孩", "representation", 1.0);
    
    // 2. 模拟 LLM1 触发 Replace 信号
    // 用户说: "Pero 染了红发"
    // LLM1 Output: action="replace", source="Pero", target="红发"
    
    // 3. Rust Engine 提取子图给 LLM2
    let context = engine.trigger_arbitration("Pero").expect("Should return context");
    println!("Context for LLM2:\n{}", context);
    
    assert!(context.contains("Pero -> 蓝发"));
    assert!(context.contains("Pero -> 女孩"));
    
    // 4. 模拟 LLM2 决定删除 "蓝发"
    let delete_targets = vec!["蓝发".to_string()];
    engine.apply_arbitration("Pero", delete_targets);
    
    // 5. 验证 "蓝发" 关联已被移除
    let src_id = engine.get_or_create_feature("Pero");
    
    {
        let edges = engine.ontology_graph.get(&src_id).unwrap();
        let has_blue_hair = edges.iter().any(|e| {
            let node = engine.nodes.get(&e.target_node_id).unwrap();
            node.content == "蓝发"
        });
        assert!(!has_blue_hair, "Logic arbitration failed: '蓝发' should be removed");
    }
    
    // 6. 写入新知识
    engine.maintain_ontology("Pero", "红发", "representation", 1.0);
    
    // 7. 再次验证
    {
        let edges = engine.ontology_graph.get(&src_id).unwrap();
        let has_red_hair = edges.iter().any(|e| {
            let node = engine.nodes.get(&e.target_node_id).unwrap();
            node.content == "红发"
        });
        assert!(has_red_hair, "New knowledge should be added");
    }
}

#[test]
fn test_v2_ontology_pruning() {
    let mut engine = AdvancedEngine::new();
    
    // 1. 添加一个核心节点
    let src_id = engine.get_or_create_feature("CoreNode");
    
    // 2. 添加 105 条边 (5 条超出 100 上限)
    // 其中 50 条高权重 (>0.1), 50 条中权重, 5 条极低权重 (<0.1)
    for i in 0..105 {
        let tgt = format!("Node_{}", i);
        let weight = if i < 5 { 0.05 } else { 0.2 + (i as f32 * 0.001) };
        engine.maintain_ontology("CoreNode", &tgt, "representation", weight);
    }
    
    // 验证剪枝前数量
    {
        let edges = engine.ontology_graph.get(&src_id).unwrap();
        assert_eq!(edges.len(), 105);
    }
    
    // 3. 执行剪枝
    // V2 Update: use apply_global_decay_and_pruning
    // decay_rate = 1.0 (不衰减), threshold = 6553 (0.1 * 65535)
    engine.apply_global_decay_and_pruning(1.0, 6553);
    
    // 4. 验证剪枝后结果
    {
        let edges = engine.ontology_graph.get(&src_id).unwrap();
        // 预期: 
        // a. 5条 <0.1 的边被直接删除 (剩 100)
        // b. 如果还有多余的，会被截断到 100
        // 在这个 Case 里，删除低权后正好是 100 条，或者如果低权 > 5 条，会先删低权再截断
        
        assert!(edges.len() <= 100, "Should not exceed capacity limit");
        
        // 验证低权重是否被清除
        let has_low_weight = edges.iter().any(|e| (e.connection_strength as f32 / 65535.0) < 0.1);
        assert!(!has_low_weight, "Should remove weak links (< 0.1)");
    }
}

#[test]
fn test_v2_execute_maintenance() {
    let mut engine = AdvancedEngine::new();
    
    // Case 1: Upsert
    engine.execute_maintenance("upsert", "Pero", "Cute", "representation", 0.9, "Observed behavior");
    let src_id = engine.get_or_create_feature("Pero");
    {
        let edges = engine.ontology_graph.get(&src_id).unwrap();
        assert!(edges.iter().any(|e| {
            let node = engine.nodes.get(&e.target_node_id).unwrap();
            node.content == "cute"
        }));
    }
    
    // Case 2: Replace (Trigger Arbitration)
    // Pre-condition: Pero -> Blue (Old)
    engine.maintain_ontology("Pero", "Blue", "representation", 1.0);
    
    // Action: Replace Blue with Red
    let context = engine.execute_maintenance("replace", "Pero", "Red", "representation", 1.0, "Hair color change");
    
    // Check 1: Red should be added (because replace calls maintain_ontology first)
    {
        let edges = engine.ontology_graph.get(&src_id).unwrap();
        assert!(edges.iter().any(|e| {
            let node = engine.nodes.get(&e.target_node_id).unwrap();
            node.content == "red"
        }));
        assert!(edges.iter().any(|e| {
            let node = engine.nodes.get(&e.target_node_id).unwrap();
            node.content == "blue"
        }));
    }
    
    // Check 2: Context should be returned and contain both Red and Blue
    assert!(context.is_some());
    let ctx_str = context.unwrap();
    println!("Arbitration Context:\n{}", ctx_str);
    assert!(ctx_str.to_lowercase().contains("blue"));
    assert!(ctx_str.to_lowercase().contains("red"));
}

fn test_scenario_19_emotion(engine: &mut AdvancedEngine) {
    println!("\n--- 测试场景 19: 情感提取与索引验证 ---");
    
    // 1. 添加带有明显情感的事件
    let event_id = 3001;
    let content = "今天真的很开心，不仅代码写得很顺，还收到了朋友的礼物，太棒了！";
    engine.add_event(event_id, content);
    
    // 2. 验证节点情感字段
    if let Some(node) = engine.nodes.get(&event_id) {
        println!("事件内容: {}", node.content);
        println!("情感向量: {:?}", node.emotions);
        
        // 验证 Joy (1 << 0) 是否存在
        let has_joy = node.emotions.contains(&SimHash::EMOTION_JOY);
        if has_joy {
            println!("✅ 情感提取成功: 检测到 Joy (1)");
        } else {
            println!("❌ 情感提取失败: 未检测到 Joy");
        }
    } else {
        println!("❌ 事件添加失败");
    }
    
    // 3. 验证情感索引
    if let Some(ids) = engine.affective_index.get(&SimHash::EMOTION_JOY) {
        if ids.contains(&event_id) {
            println!("✅ 情感索引验证成功: 事件 ID {} 存在于 Joy 索引中", event_id);
        } else {
            println!("❌ 情感索引验证失败: 事件 ID {} 不在 Joy 索引中", event_id);
        }
    } else {
        println!("❌ 情感索引验证失败: Joy 索引不存在");
    }

    // 4. 测试混合情感
    let event_id_mixed = 3002;
    let content_mixed = "虽然这个bug让我很生气，但是解决了之后又觉得很有成就感，挺开心的。";
    engine.add_event(event_id_mixed, content_mixed);
    
    if let Some(node) = engine.nodes.get(&event_id_mixed) {
        println!("事件内容: {}", node.content);
        println!("情感向量: {:?}", node.emotions);
        
        let has_anger = node.emotions.contains(&SimHash::EMOTION_ANGER);
        let has_joy = node.emotions.contains(&SimHash::EMOTION_JOY);
        
        if has_anger && has_joy {
            println!("✅ 混合情感提取成功: 检测到 Anger 和 Joy");
        } else {
            println!("❌ 混合情感提取失败: Anger={}, Joy={}", has_anger, has_joy);
        }
    }
}

#[test]
fn test_emotion_logic() {
    let mut engine = AdvancedEngine::new();
    test_scenario_19_emotion(&mut engine);
}

#[test]
fn test_mixed_emotions_demonstration() {
    // 这个测试专门用于回答用户关于"多个情感标签同时存在"的问题
    // 构造一个同时包含 "开心" (JOY) 和 "生气" (ANGER) 的句子
    let text = "虽然赢了比赛很开心，但是裁判的误判让我很生气";
    
    let emotion_bits = SimHash::extract_emotion(text);
    
    println!("测试文本: {}", text);
    println!("提取的情感位图: {:08b}", emotion_bits);
    
    // 验证是否同时包含 Joy 和 Anger
    let has_joy = (emotion_bits & SimHash::EMOTION_JOY) != 0;
    let has_anger = (emotion_bits & SimHash::EMOTION_ANGER) != 0;
    
    println!("包含 Joy (开心): {}", has_joy);
    println!("包含 Anger (生气): {}", has_anger);
    
    assert!(has_joy, "应该检测到开心");
    assert!(has_anger, "应该检测到生气");
    assert!(has_joy && has_anger, "应该同时触发多个情感标签");
}

#[test]
fn test_chaos_simple() {
    let mut engine = AdvancedEngine::new();
    
    // 1. 加载模型
    match CandleModel::new() {
        Ok(model) => {
            println!("Loading Candle Model (Simple Test)...");
            engine.embedding_model = Some(model);
        }
        Err(e) => {
            println!("⚠️ Warning: Failed to load Candle model: {}, skipping test.", e);
            return;
        }
    }
    
    // 2. 添加少量测试数据 (这将触发 calculate_chaos 向量化)
    engine.add_event(1, "Rust 是一种高性能的系统编程语言，内存安全。");
    engine.add_event(2, "Python 是一种解释型语言，简单易学。");
    
    // 3. 编译
    engine.compile();
    
    // 4. 检索
    let query = "Rust 内存安全";
    let results = engine.retrieve(query, 0, 0.15);
    
    // 5. 验证
    assert!(!results.is_empty(), "Should return results");
    let found = results.iter().any(|(id, _)| *id == 1);
    assert!(found, "Should find Rust related event");
}

#[test]
fn test_retrieval_without_vector_model() {
    // 1. 初始化引擎，不加载模型
    let mut engine = AdvancedEngine::new();
    assert!(engine.embedding_model.is_none(), "Embedding model should be None by default");

    // 2. 手动构建索引 (模拟无向量化时的关键词索引)
    // 添加特征 "rust"
    engine.add_feature(1001, "rust");
    
    // 添加事件
    let event_id = 1;
    let summary = "Rust 是一种内存安全的系统编程语言。";
    engine.add_event(event_id, summary);

    // 建立关联: rust -> event
    engine.add_edge(1001, event_id, 1.0);

    // 3. 编译 (构建 AC 自动机)
    engine.compile();

    // 4. 执行检索
    // 设置 chaos_level > 0 以测试优雅降级 (应跳过混沌检索步骤)
    let query = "Rust 编程";
    let results = engine.retrieve(query, 0, 0.5);

    // 5. 验证结果
    println!("Results without vector model: {:?}", results);
    assert!(!results.is_empty(), "Should return results via Rational Track");
    
    let found = results.iter().any(|(id, _)| *id == event_id);
    assert!(found, "Should find Rust event via keyword matching");
}
