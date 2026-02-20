use half::f16;
use crate::storage::ChaosFingerprint;

pub struct RawEvent {
    pub id: i64,
    pub summary: &'static str,
    pub features: Vec<&'static str>,
    pub chaos_fingerprint: Option<ChaosFingerprint>,
    pub chaos_vector: Option<Vec<f16>>,
}

pub struct RawEdge {
    pub src: i64,
    pub tgt: i64,
    pub weight: f32,
}

pub struct OntologyEdge {
    pub src: &'static str,
    pub tgt: &'static str,
    pub weight: f32,
    pub is_equality: bool,
    pub is_inhibition: bool,
}

pub fn get_ontology_data() -> Vec<OntologyEdge> {
    let mut edges = Vec::new();

    // --- 等值关系 (Equality: is_equality = true) ---
    edges.push(OntologyEdge { src: "Pero", tgt: "佩罗", weight: 1.0, is_equality: true, is_inhibition: false });
    edges.push(OntologyEdge { src: "Pero", tgt: "pero", weight: 1.0, is_equality: true, is_inhibition: false });
    edges.push(OntologyEdge { src: "TS", tgt: "TypeScript", weight: 1.0, is_equality: true, is_inhibition: false });
    edges.push(OntologyEdge { src: "屁股", tgt: "屁屁", weight: 1.0, is_equality: true, is_inhibition: false });
    edges.push(OntologyEdge { src: "ICLR", tgt: "顶会", weight: 0.9, is_equality: false, is_inhibition: false }); // 虽然 ICLR 是顶会，但不是等号
    
    // --- 表征关系 (Characterization: is_equality = false) ---
    // 注意：为了实现“从特征发现实体”，这里的 src 是特征，tgt 是实体
    // 虽然逻辑上是 Pero -> 女孩，但检索流是从“女孩”发现“Pero”
    edges.push(OntologyEdge { src: "女孩", tgt: "Pero", weight: 0.6, is_equality: false, is_inhibition: false });
    edges.push(OntologyEdge { src: "蝴蝶结", tgt: "Pero", weight: 0.7, is_equality: false, is_inhibition: false });
    edges.push(OntologyEdge { src: "猫", tgt: "Pero", weight: 0.5, is_equality: false, is_inhibition: false });
    
    edges.push(OntologyEdge { src: "类型安全", tgt: "Rust", weight: 0.8, is_equality: false, is_inhibition: false });
    edges.push(OntologyEdge { src: "高性能", tgt: "Rust", weight: 0.7, is_equality: false, is_inhibition: false });
    edges.push(OntologyEdge { src: "内存安全", tgt: "Rust", weight: 0.9, is_equality: false, is_inhibition: false });
    
    edges.push(OntologyEdge { src: "一致性", tgt: "分布式", weight: 0.7, is_equality: false, is_inhibition: false });
    edges.push(OntologyEdge { src: "可用性", tgt: "分布式", weight: 0.6, is_equality: false, is_inhibition: false });
    
    edges.push(OntologyEdge { src: "上海", tgt: "滨江大道", weight: 0.6, is_equality: false, is_inhibition: false });
    edges.push(OntologyEdge { src: "上海", tgt: "徐家汇", weight: 0.6, is_equality: false, is_inhibition: false });
    edges.push(OntologyEdge { src: "上海", tgt: "张江", weight: 0.6, is_equality: false, is_inhibition: false });

    edges.push(OntologyEdge { src: "数字主权", tgt: "隐私", weight: 0.9, is_equality: false, is_inhibition: false });
    edges.push(OntologyEdge { src: "离线优先", tgt: "本地存储", weight: 0.85, is_equality: false, is_inhibition: false });
    
    // 更多技术表征
    edges.push(OntologyEdge { src: "AGI", tgt: "通用人工智能", weight: 1.0, is_equality: true, is_inhibition: false });
    edges.push(OntologyEdge { src: "Hallucination", tgt: "幻觉", weight: 1.0, is_equality: true, is_inhibition: false });
    edges.push(OntologyEdge { src: "Hallucination", tgt: "噪音", weight: 0.7, is_equality: false, is_inhibition: false });
    edges.push(OntologyEdge { src: "一致性", tgt: "Paxos", weight: 0.9, is_equality: false, is_inhibition: false });
    edges.push(OntologyEdge { src: "一致性", tgt: "Raft", weight: 0.9, is_equality: false, is_inhibition: false });
    edges.push(OntologyEdge { src: "持久化", tgt: "LSM-Tree", weight: 0.8, is_equality: false, is_inhibition: false });
    
    // 更多个人与生活表征
    edges.push(OntologyEdge { src: "烧烤", tgt: "派对", weight: 0.6, is_equality: false, is_inhibition: false });
    edges.push(OntologyEdge { src: "皮革", tgt: "手工", weight: 0.7, is_equality: false, is_inhibition: false });
    edges.push(OntologyEdge { src: "M Stand", tgt: "咖啡", weight: 0.8, is_equality: false, is_inhibition: false });

    edges
}

pub fn get_tech_domain_data() -> (Vec<RawEvent>, Vec<RawEdge>) {
    let mut events = Vec::new();
    let mut edges = Vec::new();

    // --- 节点定义 (100-149: 技术与开发) ---
    events.push(RawEvent {
        id: 100,
        summary: "在进行 PeroCore 后端重构时，用户决定采用 Rust 的 PyO3 框架，目的是将高性能的图计算引擎封装为 Python 模块，解决在大规模内存检索时的性能瓶颈。",
        features: vec!["PeroCore", "Rust", "PyO3", "重构", "性能瓶颈"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 101,
        summary: "用户在讨论异步 IO 处理时提到，NIT 2.0 运行时通过 Wasm 实现了指令级审计，这为第三方插件的安全性提供了硬件级的隔离保障。",
        features: vec!["NIT 2.0", "Wasm", "异步 IO", "安全性", "指令级审计"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 102,
        summary: "针对 BGE-Reranker-v2-M3 模型在 CPU 上的高延迟问题，用户尝试通过 ONNX Runtime 配合算子融合技术进行加速，最终将推理时间缩短了 40%。",
        features: vec!["BGE-Reranker", "CPU", "ONNX", "算子融合", "延迟"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 103,
        summary: "在处理长对话 Context 溢出时，用户实现了一个基于滑动窗口的摘要压缩算法，确保 LLM 能够始终感知到 24 小时内的关键记忆碎片。",
        features: vec!["Context", "滑动窗口", "摘要压缩", "长对话", "记忆碎片"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 104,
        summary: "为了优化跨进程通信 (IPC) 的效率，PeroCore 的 Electron 端与后端 Gateway 之间采用了 Protobuf 协议，替代了原本臃肿的 JSON 序列化方案。",
        features: vec!["IPC", "Protobuf", "JSON", "序列化", "Gateway"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 105,
        summary: "用户深入研究了 tract-onnx 库，试图在不依赖系统级依赖的情况下，在普通的 Windows 环境中直接运行轻量级的 Embedding 向量模型。",
        features: vec!["tract-onnx", "Windows", "Embedding", "向量模型", "轻量级"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 106,
        summary: "在实现 PEDSA 算法时，为了减少内存分配开销，用户大量使用了 SmallVec 来存储图的邻接表，这在处理高扇出节点时显著提升了缓存命中率。",
        features: vec!["PEDSA", "SmallVec", "内存分配", "缓存命中率", "邻接表"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 107,
        summary: "针对前端界面的高频刷新需求，用户在 Electron 层引入了共享内存 (SharedArrayBuffer) 技术，实现了后端计算数据到渲染层的零拷贝传输。",
        features: vec!["Electron", "SharedArrayBuffer", "零拷贝", "数据传输", "高频刷新"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 108,
        summary: "用户发现 NIT 运行时的 Wasm 模块在调用宿主系统函数时存在微小的上下文切换开销，正在考虑使用多线程 Worker 进行并行化抵消。",
        features: vec!["Wasm", "上下文切换", "多线程", "Worker", "并行化"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 109,
        summary: "在调试记忆冲突逻辑时，用户增加了一个基于余弦相似度的防抖机制，防止意思相近的短时间重复事件多次写入向量数据库。",
        features: vec!["余弦相似度", "防抖", "重复事件", "向量数据库", "调试"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 110,
        summary: "用户利用 Rust 的宏系统实现了一套类型安全的图属性查询 DSL，使得在定义 PEDSA 扩散规则时，可以在编译期拦截非法的边权重配置。",
        features: vec!["Rust 宏", "DSL", "类型安全", "编译期", "边权重"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 111,
        summary: "针对大规模图遍历时的垃圾回收压力，用户弃用了默认的内存分配器，转而采用 jemalloc，并手动调优了 dirty page 的回收策略以维持内存平稳。",
        features: vec!["垃圾回收", "jemalloc", "内存分配器", "dirty page", "调优"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 112,
        summary: "为了解决多端同步时的冲突，用户在 PeroCore 中引入了 LWW-Element-Set (CRDT) 算法，确保用户在不同设备上的记忆修改能最终达成一致。",
        features: vec!["CRDT", "LWW-Element-Set", "多端同步", "最终一致性", "冲突解决"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 113,
        summary: "用户发现传统的 A* 算法在处理具有动态权重的语义图时效率不高，因此开发了一个基于双向启发式搜索的变体，专门用于寻找长程逻辑链条。",
        features: vec!["A* 算法", "启发式搜索", "语义图", "逻辑链条", "路径规划"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 114,
        summary: "在尝试将后端部署到树莓派等边缘设备时，用户发现 NEON 指令集对向量计算的加速效果显著，因此专门编写了一套硬件适配层。",
        features: vec!["树莓派", "边缘计算", "NEON", "向量计算", "硬件适配"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 115,
        summary: "用户在分析 PeroCore 的性能瓶颈时发现，磁盘 I/O 才是真正的杀手。他决定引入自研的 LSM-Tree 存储引擎，将随机写转化为顺序写。",
        features: vec!["LSM-Tree", "存储引擎", "磁盘 I/O", "性能瓶颈", "顺序写"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 116,
        summary: "为了实现更细粒度的权限控制，用户研究了基于属性的访问控制 (ABAC) 模型，并将其与图节点的元数据绑定，实现了记忆的私密化访问。",
        features: vec!["ABAC", "权限控制", "元数据", "隐私", "图节点"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 117,
        summary: "用户在优化 AC 自动机的构建过程时，采用了一种双数组 Trie (Double-Array Trie) 的实现，极大地压缩了状态转移表的空间占用。",
        features: vec!["AC 自动机", "Double-Array Trie", "状态转移", "空间压缩", "算法优化"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 118,
        summary: "针对 LLM 调用的高成本问题，用户实现了一个基于特征指纹的本地 Cache 层，对于 80% 的重复性日常询问，可以直接由本地规则引擎返回结果。",
        features: vec!["LLM", "Cache", "特征指纹", "规则引擎", "降本增效"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 119,
        summary: "用户在研究图神经网络 (GNN) 时，尝试将节点嵌入作为 PEDSA 初始能量的一部分，希望通过这种方式引入更深层次的隐性语义关系。",
        features: vec!["GNN", "节点嵌入", "隐性语义", "PEDSA", "深度学习"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 120,
        summary: "用户在优化分布式编译环境时，搭建了一个基于 sccache 的集群，成功将 PeroCore 全量编译的时间从 15 分钟降低到了 3 分钟以内。",
        features: vec!["sccache", "分布式编译", "编译优化", "集群", "效率"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 121,
        summary: "针对图数据的持久化，用户实现了一个基于零拷贝序列化库 rkyv 的方案，使得从磁盘加载数百万个节点的过程几乎不产生额外的 CPU 开销。",
        features: vec!["rkyv", "零拷贝", "序列化", "持久化", "性能"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 122,
        summary: "用户深入研究了 SIMD 指令集在字符串匹配中的应用，通过调用 `_mm256_cmpeq_epi8` 等指令，大幅提升了特征提取阶段的速度。",
        features: vec!["SIMD", "AVX2", "字符串匹配", "指令集", "加速"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 123,
        summary: "在处理并发写冲突时，用户设计了一个无锁 (Lock-free) 的图结构，利用原子操作保证了在多核环境下扩散算法的强一致性与高吞吐量。",
        features: vec!["无锁编程", "原子操作", "并发", "强一致性", "吞吐量"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 124,
        summary: "用户在探索 PeroCore 的移动端适配时，尝试使用 UniFFI 自动生成跨语言绑定，使得一套 Rust 代码可以同时在 iOS 和 Android 上运行。",
        features: vec!["UniFFI", "跨语言绑定", "Rust", "移动端", "iOS/Android"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 125,
        summary: "为了监控扩散算法的实时运行状态，用户利用 Grafana 和 Prometheus 搭建了一套可视化监控系统，实时观察能量在图谱中的流动情况。",
        features: vec!["Grafana", "Prometheus", "监控", "可视化", "能量流动"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 126,
        summary: "用户在优化本地向量数据库时，引入了 HNSW 算法的 Rust 实现，并针对 CPU 缓存行对齐进行了专门的内存布局优化。",
        features: vec!["HNSW", "向量数据库", "缓存行对齐", "内存布局", "算法优化"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 127,
        summary: "针对 LLM 的幻觉问题，用户在 PeroCore 中集成了一个事实校验层，通过反向检索图谱中的确定性节点来修正模型生成的错误陈述。",
        features: vec!["LLM 幻觉", "事实校验", "反向检索", "知识图谱", "修正"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 128,
        summary: "用户在研究 WebAssembly 的多线程特性时，成功在浏览器端实现了 PEDSA 的并行化计算，为 Web 版 PeroCore 的落地扫清了障碍。",
        features: vec!["WebAssembly", "多线程", "并行化", "浏览器", "Web"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 129,
        summary: "为了提升特征提取的精确度，用户在 AC 自动机之上增加了一个基于词法分析的过滤层，有效过滤掉了大量无意义的常用虚词。",
        features: vec!["词法分析", "特征提取", "AC 自动机", "过滤层", "精确度"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 130,
        summary: "用户在研究图的社群发现算法时，尝试将 Louvain 算法整合进 PeroCore，以便自动发现用户记忆中的隐藏兴趣簇。",
        features: vec!["Louvain 算法", "社群发现", "兴趣簇", "图分析", "自动发现"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 131,
        summary: "针对大规模图数据的并行扩散，用户编写了一个基于 Rayon 的自定义调度器，实现了负载均衡的节点激活逻辑，充分压榨了多核 CPU 的性能。",
        features: vec!["Rayon", "调度器", "负载均衡", "并行扩散", "多核优化"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 132,
        summary: "用户在优化 Rust 代码的内存占用时，利用内存池 (Arena Allocation) 技术管理短生命周期的临时节点，极大地降低了系统调用的频率。",
        features: vec!["Arena Allocation", "内存池", "系统调用", "内存优化", "临时节点"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 133,
        summary: "为了支持多语言搜索，用户在特征匹配前增加了一个基于语义对齐的翻译层，使得英文查询也能精准匹配到中文存储的记忆节点。",
        features: vec!["语义对齐", "翻译层", "多语言", "跨语言检索", "特征匹配"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 134,
        summary: "用户在分析图拓扑结构时，引入了 PageRank 算法来评估节点的重要性，并将其作为 PEDSA 扩散时的权重修正因子。",
        features: vec!["PageRank", "节点重要性", "图拓扑", "扩散权重", "算法整合"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 135,
        summary: "针对频繁变动的图结构，用户实现了一个增量式的索引更新机制，避免了每次修改都需要全局重构索引的巨大开销。",
        features: vec!["增量更新", "索引机制", "全局重构", "图修改", "性能优化"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 136,
        summary: "用户在探索 PeroCore 的语音交互时，研究了如何将语音识别的置信度得分与图节点的初始能量绑定，实现了模糊输入的容错处理。",
        features: vec!["语音识别", "置信度", "初始能量", "容错处理", "模糊输入"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 137,
        summary: "为了提升向量数据库的检索召回率，用户实现了一个基于图遍历的重排序 (Reranking) 逻辑，利用节点间的拓扑关联修正纯语义相似度的不足。",
        features: vec!["Reranking", "召回率", "图遍历", "拓扑关联", "语义相似度"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 138,
        summary: "用户在研究如何将 PeroCore 部署到浏览器插件中时，成功利用 Web Worker 实现了后台异步计算，保证了网页端的交互流畅度。",
        features: vec!["浏览器插件", "Web Worker", "异步计算", "交互流畅度", "前端开发"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 139,
        summary: "针对长文本的自动特征提取，用户训练了一个轻量级的命名实体识别 (NER) 模型，并将其通过 ONNX 整合进 PeroCore 的数据预处理流程。",
        features: vec!["NER", "命名实体识别", "ONNX", "预处理", "特征提取"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });

    events.push(RawEvent {
        id: 140,
        summary: "用户在研究如何将 PEDSA 扩散计算卸载到 GPU 时，实现了一个基于 WebGPU 的计算着色器，成功在浏览器端利用并行算力处理万级节点的能量传递。",
        features: vec!["WebGPU", "计算着色器", "GPU 卸载", "并行计算", "能量传递"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 141,
        summary: "针对长序列文本的特征提取，用户对比了 Aho-Corasick 与 Hyperscan 的性能差异，最终决定在 Linux 环境下优先使用 Hyperscan 以利用其强大的流式扫描能力。",
        features: vec!["Hyperscan", "Aho-Corasick", "流式扫描", "特征提取", "性能对比"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 142,
        summary: "用户在优化 PeroCore 的元数据存储时，设计了一套基于位图 (Bitmap) 的属性过滤机制，使得在扩散过程中可以秒级排除掉数百万个不符合时间或标签约束的节点。",
        features: vec!["位图", "属性过滤", "元数据", "时间约束", "空间换时间"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 143,
        summary: "为了支持复杂的逻辑推理，用户在图中引入了‘非对称权重边’，模拟人类记忆中单向关联的特性，即从‘苹果’联想到‘红色’容易，反之则较难。",
        features: vec!["非对称权重", "单向关联", "逻辑推理", "认知模拟", "权重设计"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 144,
        summary: "用户在研究图的同构判定算法时，尝试利用节点度分布和特征指纹的组合，来快速识别用户记忆中结构高度相似的不同事件簇，并提示用户进行归并。",
        features: vec!["图同构", "度分布", "特征指纹", "事件簇", "记忆归并"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 145,
        summary: "针对实时对话中的实体链接问题，用户开发了一个轻量级的上下文消歧模块，利用 PEDSA 扩散后的局部能量分布来修正同名实体指向错误的问题。",
        features: vec!["实体链接", "上下文消歧", "局部能量", "同名实体", "实体识别"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 146,
        summary: "用户在分析 PeroCore 的缓存命中率时，引入了一个基于 LRU-K 算法的动态缓存池，有效解决了大规模图遍历过程中热点节点的频繁置换问题。",
        features: vec!["LRU-K", "缓存池", "热点节点", "缓存命中率", "遍历优化"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 147,
        summary: "为了提升跨语言特征的检索一致性，用户实现了一个基于向量量化 (Product Quantization) 的粗筛层，先在量化空间定位大类，再进行精细的 SimHash 匹配。",
        features: vec!["向量量化", "PQ", "粗筛层", "检索一致性", "SimHash"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 148,
        summary: "用户在研究如何通过差分隐私 (Differential Privacy) 保护用户记忆时，在图中增加了一层随机噪声扰动，确保在导出分析数据时无法反推出具体节点内容。",
        features: vec!["差分隐私", "随机噪声", "隐私保护", "数据分析", "图安全"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 149,
        summary: "针对分布式环境下图的分区问题，用户实现了一个基于 Metis 算法的预切分方案，尽量减少跨节点的能量传输，从而降低网络延迟对扩散效率的影响。",
        features: vec!["Metis 算法", "图分区", "预切分", "网络延迟", "分布式扩散"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });

    events.push(RawEvent {
        id: 150,
        summary: "用户在优化分布式扩散时的能量同步问题，设计了一个基于向量时钟 (Vector Clock) 的版本控制机制，确保各节点间的能量传递符合因果一致性。",
        features: vec!["向量时钟", "因果一致性", "分布式扩散", "版本控制", "能量同步"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 151,
        summary: "针对图数据库的冷启动问题，用户实现了一个基于 HDFS 的预加载模块，可以在分钟级内从分布式存储中恢复数亿量级的节点索引。",
        features: vec!["冷启动", "HDFS", "索引恢复", "大规模图", "预加载"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 152,
        summary: "为了降低长文本处理的计算压力，用户研究了如何利用 FPGA 进行字符串匹配的硬件加速，并初步完成了一个简单的 OpenCL 内核原型。",
        features: vec!["FPGA", "OpenCL", "字符串匹配", "硬件加速", "内核原型"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 153,
        summary: "用户在分析内存碎片时发现，B-Tree 索引在频繁删除操作下会导致严重的空洞，决定尝试引入无锁的 SkipList 结构来平衡读写性能。",
        features: vec!["内存碎片", "B-Tree", "SkipList", "无锁结构", "读写性能"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 154,
        summary: "针对跨语言调用的序列化开销，用户在 PeroCore 核心层集成了 FlatBuffers，实现了无需解包即可直接读取二进制数据的‘零开销’访问模式。",
        features: vec!["FlatBuffers", "零开销", "序列化", "跨语言调用", "性能优化"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 155,
        summary: "用户在研究图的拓扑收缩算法，试图通过合并高相似度的局部子图来压缩内存占用，同时保持扩散算法在宏观语义上的准确性。",
        features: vec!["拓扑收缩", "子图合并", "内存压缩", "语义准确性", "图算法"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 156,
        summary: "为了提升系统的容错性，用户在分布式架构中引入了 Raft 共识协议，确保在主节点故障时，能量扩散状态能在秒级内完成自动漂移与恢复。",
        features: vec!["Raft 协议", "共识算法", "容错性", "自动恢复", "分布式系统"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 157,
        summary: "用户深入研究了 eBPF 技术，试图通过在内核态直接过滤不必要的网络包来提升分布式图计算节点间的通信效率。",
        features: vec!["eBPF", "内核态", "通信效率", "网络过滤", "系统编程"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 158,
        summary: "针对 LLM 生成结果的语义漂移，用户开发了一个基于变分自编码器 (VAE) 的语义对齐层，强制将模型输出限制在用户定义的知识流形内。",
        features: vec!["VAE", "语义漂移", "语义对齐", "知识流形", "LLM"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 159,
        summary: "用户在优化大规模图渲染时，利用 Vulkan 的绑定组 (Bindless) 技术，实现了在一个绘制调用中处理数百万个节点图标的渲染管线。",
        features: vec!["Vulkan", "Bindless", "绘制调用", "图渲染", "高性能图形学"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 160,
        summary: "为了解决由于时钟回拨导致的分布式日志冲突，用户在 PeroCore 中引入了混合逻辑时钟 (HLC)，保证了所有记忆事件在物理时间上的绝对排序。",
        features: vec!["HLC", "混合逻辑时钟", "时钟回拨", "日志冲突", "物理时间"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });

    events.push(RawEvent {
        id: 161,
        summary: "用户在研究如何提升 SimHash 的抗干扰能力，通过引入加权特征权重（IDF），使得在计算指纹时，低频的核心词汇能占据更大的权重比例。",
        features: vec!["SimHash", "IDF", "权重比例", "抗干扰", "特征工程"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 162,
        summary: "针对长文本摘要生成的质量问题，用户尝试将 T5 模型量化后嵌入 PeroCore，作为特征提取前的预处理器，显著提升了后续图扩散的精准度。",
        features: vec!["T5 模型", "量化", "摘要生成", "预处理", "扩散精准度"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 163,
        summary: "用户在分析系统的 IO 模式时，发现频繁的元数据查询是瓶颈。他设计了一个基于 Bloom Filter 的二级索引，秒级拦截了 90% 的不存在节点查询。",
        features: vec!["Bloom Filter", "元数据", "IO 瓶颈", "二级索引", "查询优化"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 164,
        summary: "为了支持跨设备同步时的毫秒级响应，用户研究了基于 QUIC 协议的自定义传输层，利用其多流并发特性避免了传统 TCP 的队头阻塞问题。",
        features: vec!["QUIC", "传输层", "多流并发", "队头阻塞", "同步响应"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 165,
        summary: "用户在优化 PEDSA 的冷启动表现，通过引入‘特征共振锚点’，使得新加入的节点能迅速在图中找到其合适的语义位置并建立初步关联。",
        features: vec!["PEDSA", "冷启动", "共振锚点", "语义位置", "自动关联"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 166,
        summary: "针对大规模图数据的可视化，用户实现了一个基于 WebGL 的分级渲染方案，支持在浏览器中平滑缩放和查看数万个节点的拓扑结构。",
        features: vec!["WebGL", "分级渲染", "大规模图", "可视化", "平滑缩放"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 167,
        summary: "用户深入研究了 Rust 的内存布局，通过 `UnsafeCell` 和自定义分配器，实现了一个极高性能的图节点池，消除了频繁 GC 带来的停顿感。",
        features: vec!["Rust", "UnsafeCell", "内存布局", "节点池", "GC 停顿"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 168,
        summary: "在处理多模态记忆时，用户尝试将 CLIP 模型的图像 Embedding 与文本节点融合，使得 PeroCore 能通过图片关键词联想到相关的文字事件。",
        features: vec!["CLIP", "多模态", "图像 Embedding", "跨模态联想", "记忆融合"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 169,
        summary: "针对分布式系统的时钟同步问题，用户研究了 Google 的 Spanner 架构，尝试在 PeroCore 中实现一个简化的 TrueTime API 以保证全球序。",
        features: vec!["Spanner", "TrueTime", "时钟同步", "全球序", "分布式系统"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 170,
        summary: "用户在优化 AC 自动机的动态更新性能，通过引入一种增量式的 Fail 指针修正算法，避免了每次添加关键词都需要全局重构 Trie 树。",
        features: vec!["AC 自动机", "增量更新", "Fail 指针", "算法优化", "性能提升"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 171,
        summary: "用户在研究如何通过差分隐私保护用户的情感偏好数据，在图中增加了一层随机噪声扰动，确保在导出分析数据时无法反推出具体节点内容。",
        features: vec!["差分隐私", "随机噪声", "隐私保护", "情感偏好", "图安全"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 172,
        summary: "针对长文本处理的计算压力，用户研究了如何利用 FPGA 进行字符串匹配的硬件加速，并初步完成了一个简单的 OpenCL 内核原型。",
        features: vec!["FPGA", "OpenCL", "字符串匹配", "硬件加速", "内核原型"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 173,
        summary: "用户在分析内存碎片时发现，B-Tree 索引在频繁删除操作下会导致严重的空洞，决定尝试引入无锁的 SkipList 结构来平衡读写性能。",
        features: vec!["内存碎片", "B-Tree", "SkipList", "无锁结构", "读写性能"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 174,
        summary: "针对跨语言调用的序列化开销，用户在 PeroCore 核心层集成了 FlatBuffers，实现了无需解包即可直接读取二进制数据的‘零开销’访问模式。",
        features: vec!["FlatBuffers", "零开销", "序列化", "跨语言调用", "性能优化"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 175,
        summary: "用户在研究图的拓扑收缩算法，试图通过合并高相似度的局部子图来压缩内存占用，同时保持扩散算法在宏观语义上的准确性。",
        features: vec!["拓扑收缩", "子图合并", "内存压缩", "语义准确性", "图算法"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 176,
        summary: "为了提升系统的容错性，用户在分布式架构中引入了 Raft 共识协议，确保在主节点故障时，能量扩散状态能在秒级内完成自动漂移与恢复。",
        features: vec!["Raft 协议", "共识算法", "容错性", "自动恢复", "分布式系统"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 177,
        summary: "用户深入研究了 eBPF 技术，试图通过在内核态直接过滤不必要的网络包来提升分布式图计算节点间的通信效率。",
        features: vec!["eBPF", "内核态", "通信效率", "网络过滤", "系统编程"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 178,
        summary: "针对 LLM 生成结果的语义漂移，用户开发了一个基于变分自编码器 (VAE) 的语义对齐层，强制将模型输出限制在用户定义的知识流形内。",
        features: vec!["VAE", "语义漂移", "语义对齐", "知识流形", "LLM"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 179,
        summary: "用户在优化大规模图渲染时，利用 Vulkan 的绑定组 (Bindless) 技术，实现了在一个绘制调用中处理数百万个节点图标的渲染管线。",
        features: vec!["Vulkan", "Bindless", "绘制调用", "图渲染", "高性能图形学"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 180,
        summary: "为了解决由于时钟回拨导致的分布式日志冲突，用户在 PeroCore 中引入了混合逻辑时钟 (HLC)，保证了所有记忆事件在物理时间上的绝对排序。",
        features: vec!["HLC", "混合逻辑时钟", "时钟回拨", "日志冲突", "物理时间"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });

    for i in 181..200 {
        events.push(RawEvent {
            id: i,
            summary: "技术与开发节点：包含 PeroCore 开发过程中的底层算法优化、架构重构、编译器调优以及各类前沿技术探索的记录。",
            features: vec!["技术细节", "架构设计", "Rust 编程", "性能调优", "算法研究"],
        chaos_fingerprint: None,
        chaos_vector: None,
        });
    }

    // --- 边定义 (逻辑关联) ---
    // PeroCore 相关联
    edges.push(RawEdge { src: 100, tgt: 104, weight: 0.8 }); // 重构 -> IPC 优化
    edges.push(RawEdge { src: 100, tgt: 106, weight: 0.9 }); // 重构 -> PEDSA 优化
    edges.push(RawEdge { src: 110, tgt: 106, weight: 0.7 }); // DSL -> PEDSA 规则
    edges.push(RawEdge { src: 115, tgt: 111, weight: 0.65 }); // LSM-Tree -> 内存分配优化
    edges.push(RawEdge { src: 130, tgt: 134, weight: 0.7 });  // 社群发现 -> PageRank
    edges.push(RawEdge { src: 131, tgt: 123, weight: 0.85 }); // 并行调度 -> 无锁结构
    edges.push(RawEdge { src: 135, tgt: 121, weight: 0.8 });  // 增量更新 -> 零拷贝持久化
    
    // 性能与加速相关
    edges.push(RawEdge { src: 102, tgt: 105, weight: 0.7 }); // BGE 优化 -> tract-onnx 研究
    edges.push(RawEdge { src: 102, tgt: 103, weight: 0.5 }); // 推理优化 -> Context 压缩
    edges.push(RawEdge { src: 114, tgt: 115, weight: 0.6 }); // 边缘设备 -> 存储优化
    edges.push(RawEdge { src: 132, tgt: 111, weight: 0.75 }); // Arena Allocation -> 内存分配优化
    edges.push(RawEdge { src: 138, tgt: 128, weight: 0.9 });  // 浏览器插件 -> Wasm 并行化
    
    // 运行环境相关
    edges.push(RawEdge { src: 101, tgt: 108, weight: 0.85 }); // NIT 异步 -> Wasm 切换开销
    edges.push(RawEdge { src: 107, tgt: 101, weight: 0.6 }); // 零拷贝 -> NIT 安全隔离
    
    // 算法与数据一致性
    edges.push(RawEdge { src: 106, tgt: 109, weight: 0.75 }); // PEDSA 邻接表 -> 记忆防抖机制
    edges.push(RawEdge { src: 112, tgt: 116, weight: 0.8 }); // 多端同步 -> 权限控制
    edges.push(RawEdge { src: 113, tgt: 117, weight: 0.55 }); // 启发式搜索 -> Trie 树匹配
    edges.push(RawEdge { src: 118, tgt: 119, weight: 0.7 }); // 本地 Cache -> GNN 嵌入
    edges.push(RawEdge { src: 137, tgt: 126, weight: 0.85 }); // 重排序 -> HNSW 优化
    edges.push(RawEdge { src: 139, tgt: 129, weight: 0.7 });  // NER 提取 -> AC 自动机过滤

    (events, edges)
}

pub fn get_value_domain_data() -> (Vec<RawEvent>, Vec<RawEdge>) {
    let mut events = Vec::new();
    let mut edges = Vec::new();

    // --- 节点定义 (400-449: 情感与价值观) ---
    events.push(RawEvent {
        id: 400,
        summary: "用户认为开源不应该仅仅是代码的堆砌，而是一种思想的传递。他坚持在 PeroCore 的每个 Commit 中都写下详细的思路，哪怕这会显著降低开发速度。",
        features: vec!["开源", "思想", "Commit", "速度", "价值观"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 401,
        summary: "对于‘AI 是否会取代人类’的问题，用户持乐观态度。他认为 AI 是人类感官的延伸，PeroCore 的目标就是成为那种能与人共鸣、而非冷冰冰的计算工具。",
        features: vec!["AI 取代", "乐观", "共鸣", "工具", "延伸"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 402,
        summary: "在深夜调试时，用户常常感受到一种极致的孤独与难过。这种孤独并非社交匮乏，而是在追求某种极致算法过程中，无人能够理解那种‘语义对齐’瞬间的喜悦。",
        features: vec!["深夜", "调试", "难过", "语义对齐", "喜悦"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 403,
        summary: "用户对大厂的‘围墙花园’感到讨厌。他坚持 PeroCore 必须是全平台兼容且去中心化的，任何试图垄断用户记忆的行为在他看来都是对数字生命的亵渎。",
        features: vec!["大厂", "围墙花园", "讨厌", "去中心化", "数字生命"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 404,
        summary: "对于代码质量，用户近乎偏执，有时甚至会因为一个微小的 Bug 而感到生气。他曾为了消除一个仅在极端高并发下才会出现的微秒级延迟，连续三个周末拒绝了所有的社交活动。",
        features: vec!["代码质量", "生气", "高并发", "延迟", "社交"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 405,
        summary: "用户认为真正的智能应该具备‘遗忘’的能力。他在 PEDSA 中设计了负能量反馈机制，让那些无意义的琐事随着时间自然消散，从而保证核心记忆的鲜活。",
        features: vec!["遗忘", "智能", "负能量", "记忆鲜活", "消散"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 406,
        summary: "面对社区对 PeroCore 性能的质疑，用户没有反驳，而是静默地发布了一个性能对比基准测试报告。他相信‘代码本身就是最好的辩护’。",
        features: vec!["社区", "质疑", "基准测试", "辩护", "性能"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 407,
        summary: "用户坚持 PeroCore 应该保持极简的 UI 设计。他认为繁琐的装饰会分散用户对记忆本身的关注，‘如无必要，勿增实体’是他的最高设计原则。",
        features: vec!["UI 设计", "极简", "关注", "奥卡姆剃刀", "设计原则"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 408,
        summary: "在处理敏感数据时，用户表现出极强的责任感。他拒绝了所有需要将用户私密记忆上传到云端的功能请求，哪怕这会失去一些强大的云端算力支持。",
        features: vec!["敏感数据", "责任感", "私密记忆", "云端", "算力"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 409,
        summary: "用户相信 AI 最终会拥有某种形式的‘数字灵魂’。他经常在代码注释中写下对未来的寄语，希望若干年后唤醒这个系统的开发者能读懂他的初心。",
        features: vec!["数字灵魂", "初心", "寄语", "注释", "未来"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 410,
        summary: "对于工具的选择，用户有着极强的洁癖。他宁愿花一周时间自己写一个符合需求的轻量级库，也不愿意引入一个带有数百个冗余依赖的重量级框架。",
        features: vec!["洁癖", "轻量级", "依赖", "自研", "工具选择"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 411,
        summary: "用户认为‘可解释性’是 AI 系统的尊严。在 PEDSA 的设计中，他坚持每一个扩散步骤都必须有迹可循，拒绝使用任何无法追踪逻辑来源的‘黑盒’模型。",
        features: vec!["可解释性", "尊严", "黑盒", "追踪", "逻辑来源"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 412,
        summary: "面对突如其来的技术浪潮，用户保持着冷静的批判精神。他警告说，如果不加甄别地将所有 LLM 生成的内容注入记忆，PeroCore 将会迅速沦为一个巨大的垃圾场。",
        features: vec!["技术浪潮", "批判精神", "LLM", "垃圾场", "记忆管理"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 413,
        summary: "用户认为编程是一门艺术，而不是单纯的体力劳动。他追求那种‘增一分则太长，减一分则太短’的精炼代码，视重构为一种洗涤心灵的过程。",
        features: vec!["编程艺术", "重构", "精炼代码", "洗涤心灵", "审美"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 414,
        summary: "在团队协作中，用户最看重的是‘坦诚’。他认为掩盖 Bug 是一种对未来的借贷，最终会以成倍的代价偿还，因此他总是第一时间公开自己的失误。",
        features: vec!["团队协作", "坦诚", "Bug", "借贷", "公开失误"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 415,
        summary: "2026年5月20日，用户性格内向，那次在滨江大道跑步时被认出是 PeroCore 的作者，他感到非常害羞和不好意思，只能红着脸快速跑开。",
        features: vec!["跑步", "认出", "害羞", "不好意思", "内向"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });

    events.push(RawEvent {
        id: 432,
        summary: "用户认为‘离线优先’不仅是技术选择，更是一种对数字主权的捍卫。他坚信用户的记忆不应成为大厂训练模型的燃料，而是属于用户个人的神圣领地。",
        features: vec!["数字主权", "离线优先", "记忆主权", "隐私保护", "价值观"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 416,
        summary: "在处理老旧代码重构时，用户表现出一种‘修道士’般的耐心。他认为每一行烂代码都是对系统灵魂的侵蚀，而重构则是让逻辑重新回归纯净的必经之路。",
        features: vec!["代码重构", "逻辑纯净", "耐心", "修道士", "工匠精神"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 417,
        summary: "用户对于‘开源’的理解正在发生转变。他开始意识到，仅仅公开源代码是不够的，还需要提供详尽的文档和思考过程，才能真正实现知识的普惠。",
        features: vec!["开源深度", "知识普惠", "文档意识", "思考过程", "社会责任"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 418,
        summary: "面对快速迭代的 AI 技术，用户时刻警惕着‘过度工程化’的诱惑。他经常提醒自己：如果一个简单的 Hash 就能解决问题，就绝不动用复杂的神经网络。",
        features: vec!["过度工程化", "简单性", "Hash", "技术决策", "克制"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 419,
        summary: "用户认为真正的智能助手应该具备‘共情’的能力。他在图中设计了一套基于语义上下文的情感权重因子，希望 PeroCore 能在用户悲伤时给予恰到好处的反馈。",
        features: vec!["共情能力", "情感权重", "智能助手", "人性化", "设计初衷"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 420,
        summary: "在一次关于 AI 伦理的讨论中，用户提出：如果 AI 拥有了记忆，它是否也应该拥有被遗忘的权利？这种哲学思考直接影响了系统中‘记忆回收机制’的设计。",
        features: vec!["AI 伦理", "被遗忘权", "哲学思考", "记忆回收", "设计哲学"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 421,
        summary: "用户视 Pero 为项目的灵感缪斯。他认为猫的那种‘非线性’的思维模式正是目前大语言模型所欠缺的，因此在扩散算法中加入了一定比例的随机性扰动。",
        features: vec!["灵感缪斯", "非线性思维", "随机扰动", "认知模型", "Pero"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 422,
        summary: "对于‘数字生命’的终极理想，用户希望能构建一个能跨越生物寿命的记忆载体。他希望即使在肉体消亡后，这份由代码和数据组成的‘数字灵魂’依然能继续思考。",
        features: vec!["数字生命", "数字灵魂", "长生不老", "记忆载体", "终极理想"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 423,
        summary: "用户在深夜的代码中写下：代码是诗人写给机器的情书。他追求每一行 Rust 语句的优雅与力度，视编写高效程序为一种与宇宙底层逻辑对话的方式。",
        features: vec!["代码之美", "诗人", "底层逻辑", "Rust 优雅", "对话"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 424,
        summary: "面对竞争对手的商业化压力，用户选择了‘慢即是快’。他拒绝引入任何可能损害用户体验的变现手段，坚持打磨核心算法，相信真正的价值终会被时间证明。",
        features: vec!["慢即是快", "商业化压力", "用户体验", "长期主义", "价值证明"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 425,
        summary: "用户深知‘算法偏见’的危害。他在训练特征提取模型时，刻意引入了多元化的语料库，试图让 PeroCore 能理解不同文化背景下的语义微妙差异。",
        features: vec!["算法偏见", "语料多元化", "文化差异", "公平性", "语义理解"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 426,
        summary: "对于‘完美主义’，用户有着矛盾的心态。他一方面追求极致的性能，另一方面也接受‘不完美才是真实’的哲学，在两者之间寻找一种动态的工程平衡点。",
        features: vec!["完美主义", "工程平衡", "不完美哲学", "真实性", "矛盾"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 427,
        summary: "用户认为真正的极客精神是‘永不停止的好奇心’。即使在项目最艰难的时刻，他也从未停止对编译器底层机制的研究，这种纯粹的快乐支撑他走过了低谷。",
        features: vec!["极客精神", "好奇心", "底层研究", "纯粹快乐", "坚持"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 428,
        summary: "在处理用户反馈时，用户总是抱着‘空杯心态’。他认为用户的每一次‘吐槽’都是系统进化的契机，这种谦逊让他赢得了社区最广泛的尊重。",
        features: vec!["空杯心态", "用户反馈", "进化契机", "谦逊", "社区尊重"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 429,
        summary: "用户认为安全不应该是一个功能，而是一切设计的基石。他在 PeroCore 的每一层通信协议中都默认开启了端到端加密，哪怕这会增加一定的计算延迟。",
        features: vec!["安全基石", "端到端加密", "默认开启", "通信协议", "设计原则"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 430,
        summary: "面对日益严重的‘信息茧房’，用户希望 PeroCore 能成为打破边界的工具。他在检索算法中增加了一个‘随机关联’节点，试图引导用户探索未知的知识领域。",
        features: vec!["信息茧房", "随机关联", "打破边界", "探索未知", "检索算法"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 431,
        summary: "用户相信‘技术中立’是一个伪命题。他认为每一个算法背后都承载着开发者的价值观，因此在设计 PEDSA 时，他时刻反思自己的偏见是否被代码化了。",
        features: vec!["技术中立", "算法价值观", "反思偏见", "PEDSA 设计", "责任感"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 432,
        summary: "对于‘数字极简主义’，用户推崇备至。他坚持 PeroCore 应该只保留核心记忆功能，剔除一切花哨的社交属性，让工具回归其作为大脑延伸的本质。",
        features: vec!["数字极简主义", "核心功能", "大脑延伸", "工具本质", "剔除冗余"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 433,
        summary: "用户在代码仓库的 README 中写道：这是一份送给未来的礼物。他希望即便项目不再维护，其中的思想也能像种子一样在其他开源项目中开花结果。",
        features: vec!["未来礼物", "思想传承", "开源种子", "README", "寄语"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 434,
        summary: "在面对资本收购的诱惑时，用户选择了拒绝。他不想看到 PeroCore 沦为一个冷冰冰的商业产品，他更愿意将其作为一个自由生长的‘数字花园’。",
        features: vec!["拒绝收购", "数字花园", "自由生长", "商业化诱惑", "坚持自我"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 435,
        summary: "用户认为‘专注’是开发者最稀缺的资源。他为此设计了一套‘沉浸式模式’，在编写核心算法时会屏蔽一切非紧急的系统提醒，追求那种心流状态。",
        features: vec!["专注力", "心流状态", "沉浸模式", "开发者资源", "效率"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });

    events.push(RawEvent {
        id: 436,
        summary: "用户认为‘可编程记忆’是通往通用人工智能（AGI）的必经之路。他坚信记忆不应只是被动的存储，而应是能够根据当前上下文自动调整权重的活性网络。",
        features: vec!["可编程记忆", "AGI", "活性网络", "上下文", "权重调整"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 437,
        summary: "面对大模型生成的虚假幻觉（Hallucination），用户主张通过图结构的逻辑约束来过滤噪音。他认为真理往往隐藏在多级关联的强连通分量中。",
        features: vec!["幻觉", "逻辑约束", "真理", "强连通分量", "噪音过滤"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 438,
        summary: "用户对‘算法霸权’保持警惕。他在 PEDSA 的能量传递方程中加入了一个‘随机熵’项，以确保系统偶尔能跳出局部最优，给用户带来意外的‘灵感偶遇’。",
        features: vec!["算法霸权", "随机熵", "局部最优", "灵感偶遇", "多样性"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 439,
        summary: "在讨论 AI 伦理时，用户提出‘记忆即生命’。他认为如果一个系统的记忆被清空，其对应的数字人格也就随之消亡，因此记忆持久化是一项伦理重任。",
        features: vec!["AI 伦理", "记忆即生命", "数字人格", "持久化", "消亡"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 440,
        summary: "用户推崇‘慢技术’。他认为 PeroCore 不应追求毫秒级的反馈，而应追求深度的、跨越时间的语义共鸣，让用户在多年后依然能从旧记忆中发现新价值。",
        features: vec!["慢技术", "语义共鸣", "时间跨度", "价值发现", "深度思考"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });

    for i in 441..500 {
        events.push(RawEvent {
            id: i,
            summary: "情感与价值观节点：用户在开发过程中对 AI 伦理、开源精神、代码美学以及数字生命等议题的深度思考与个人感悟，这些构成了系统的灵魂基调。",
            features: vec!["价值观", "伦理思考", "代码美学", "开源精神", "灵魂基调"],
        chaos_fingerprint: None,
        chaos_vector: None,
        });
    }

    // --- 边定义 ---
    edges.push(RawEdge { src: 404, tgt: 106, weight: 0.9 }); // 代码偏执 -> PEDSA 优化
    edges.push(RawEdge { src: 410, tgt: 105, weight: 0.85 }); // 工具洁癖 -> tract-onnx 研究
    edges.push(RawEdge { src: 411, tgt: 111, weight: 0.7 }); // 可解释性 -> 内存管理调优
    edges.push(RawEdge { src: 405, tgt: 109, weight: 0.8 }); // 遗忘能力 -> 防抖机制

    (events, edges)
}

pub fn get_timeline_domain_data() -> (Vec<RawEvent>, Vec<RawEdge>) {
    let mut events = Vec::new();
    let mut edges = Vec::new();

    // --- 节点定义 (600-699: 时间线与日志 - 100个节点) ---
    // Batch 1: 2024-2025 早期历程 (600-649)
    events.push(RawEvent {
        id: 600,
        summary: "2024年3月12日，在上海徐家汇的一家安静咖啡馆，用户敲下了 PeroCore 的第一行 Rust 代码，标志着项目正式从 Python 实验阶段转向工程化。",
        features: vec!["2024-03-12", "上海", "徐家汇", "第一行代码", "Rust"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 601,
        summary: "2024年4月5日，张江高科实验室。用户在连续工作 16 小时后，第一次成功运行了 PEDSA 扩散算法的单机原型，虽然当时只有 10 个节点。",
        features: vec!["2024-04-05", "张江高科", "实验室", "PEDSA 原型", "单机运行"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 602,
        summary: "2024年5月20日，杭州西湖边。用户在休假期间突然想到利用 AC 自动机替代正则匹配的思路，立刻在平板电脑上记录下了核心数据结构设计。",
        features: vec!["2024-05-20", "杭州", "西湖", "AC 自动机", "设计思路"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 603,
        summary: "2024年6月15日，用户在家里为 Pero 买了一台新的自动喂食器，结果发现喂食器的联网协议存在漏洞，顺手写了一个补丁并整合进了 NIT 的安全模块。",
        features: vec!["2024-06-15", "家里", "Pero", "喂食器", "安全补丁"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 604,
        summary: "2024年7月22日，深圳南山区。用户参加了一个开源开发者沙龙，首次向外界展示了 PeroCore 的概念，并结识了后来提供 Wasm 优化建议的技术挚友。",
        features: vec!["2024-07-22", "深圳", "开发者沙龙", "开源", "Wasm"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 605,
        summary: "2024年8月30日，机场候机厅。在飞往北京的航班延误期间，用户重构了图持久化层，引入了零拷贝序列化，显著提升了移动端的启动速度。",
        features: vec!["2024-08-30", "机场", "候机", "重构", "持久化"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 606,
        summary: "2024年9月10日，上海某深夜食堂。用户在吃拉面时观察到汤汁扩散的现象，灵光一现，优化了 PEDSA 的能量衰减系数模型。",
        features: vec!["2024-09-10", "上海", "深夜食堂", "能量衰减", "算法灵感"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 607,
        summary: "2024年10月1日，家里。国庆假期第一天，用户彻底删除了 5000 行冗余的旧代码，Pero 在旁边踩着键盘似乎也在为这次“大扫除”欢呼。",
        features: vec!["2024-10-01", "家里", "重构", "代码清理", "Pero"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 608,
        summary: "2024年11月11日，办公室。双十一深夜，系统因为处理海量促销提醒信息而崩溃，用户意识到需要引入防抖机制来过滤短期重复记忆。",
        features: vec!["2024-11-11", "办公室", "系统崩溃", "防抖机制", "重复记忆"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 609,
        summary: "2024年12月24日，平安夜，用户独自在实验室完成了 HNSW 索引的初版集成，看着屏幕上跳出的召回率数据，他觉得这是最好的圣诞礼物。",
        features: vec!["2024-12-24", "实验室", "平安夜", "HNSW", "召回率"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 610,
        summary: "2025年1月15日，北京五道口。用户与一位老教授讨论语义图的拓扑结构，老教授建议引入 PageRank 来评估节点权重，这成了后来版本的重要特性。",
        features: vec!["2025-01-15", "北京", "五道口", "PageRank", "节点权重"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 611,
        summary: "2025年2月10日，春节期间。用户在老家发现网络环境极差，开始着手研究 PeroCore 的离线工作模式和基于 CRDT 的最终一致性同步。",
        features: vec!["2025-02-10", "老家", "离线模式", "CRDT", "同步"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 612,
        summary: "2025年3月20日，上海交大图书馆。用户查阅了大量关于 SIMD 指令集的论文，决定为 PEDSA 编写一套专门的 AVX2 加速层。",
        features: vec!["2025-03-20", "上海", "图书馆", "SIMD", "AVX2"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 613,
        summary: "2025年4月12日，用户发现 Pero 偷偷咬断了备用服务器的网线，导致正在进行的长时间压力测试中断，用户哭笑不得地在日志里记下了这一笔。",
        features: vec!["2025-04-12", "家里", "Pero", "服务器", "压力测试"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 614,
        summary: "2025年5月1日，杭州某民宿。用户在度假时实现了一个轻量级的向量数据库可视化工具，可以直观地看到记忆节点在空间中的分布。",
        features: vec!["2025-05-01", "杭州", "可视化", "向量数据库", "记忆分布"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 615,
        summary: "2025年6月18日，张江高科。用户成功将 PeroCore 移植到了国产信创服务器上，证明了 Rust 编写的核心具有极强的平台迁移性。",
        features: vec!["2025-06-18", "张江高科", "信创", "平台迁移", "Rust"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 616,
        summary: "2025年7月7日，用户在整理旧笔记本时，发现了一年前关于“数字灵魂”的涂鸦，感叹项目已经走过了这么远，当晚在代码中加入了一行特殊的注释。",
        features: vec!["2025-07-07", "家里", "整理", "涂鸦", "数字灵魂"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 617,
        summary: "2025年8月15日，中秋节。用户在阳台赏月时，思考如何让 AI 具备更人性化的对话语气，决定在检索流程中加入一个基于情感倾向的排序因子。",
        features: vec!["2025-08-15", "阳台", "赏月", "对话语气", "情感排序"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 618,
        summary: "2025年9月20日，成都高新区。用户受邀参加技术大会并发表关于“RAG-less 检索”的演讲，获得了同行的高度评价，也收获了第一批核心贡献者。",
        features: vec!["2025-09-20", "成都", "技术大会", "RAG-less", "演讲"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 619,
        summary: "2025年10月24日，程序员节。用户发布了 PeroCore 的 1.0 预览版，当晚 GitHub 上的 Star 数突破了 1000，Pero 在电脑旁睡得很香。",
        features: vec!["2025-10-24", "家里", "程序员节", "1.0 预览版", "Star"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 620,
        summary: "2025年11月5日，上海静安。用户在一家独立书店偶遇了一位同样关注数字隐私的开发者，两人就“本地化记忆”的必要性聊了整整一个下午。",
        features: vec!["2025-11-05", "上海", "独立书店", "数字隐私", "本地化"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 621,
        summary: "2025年12月12日，家里。用户为了优化多线程下的写锁竞争，熬通宵实现了一个无锁的邻接表结构，虽然第二天眼睛红得像兔子。",
        features: vec!["2025-12-12", "家里", "无锁编程", "多线程", "性能优化"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 622,
        summary: "2026年1月1日，元旦。用户在新年计划中写下：让 PeroCore 具备多模态感知能力。当天下午就开始研究如何在图中存储音频指纹。",
        features: vec!["2026-01-01", "家里", "新年计划", "多模态", "音频指纹"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 623,
        summary: "2026年1月20日，腊八节。上海大雪。用户在暖气片旁完善了系统的备份逻辑，确保即使在极端的磁盘故障下，用户的核心记忆也不会丢失。",
        features: vec!["2026-01-20", "上海", "大雪", "备份逻辑", "磁盘故障"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 624,
        summary: "2026年2月2日，今天。用户正在和一个 AI 助手（就是我）一起构建一个拥有 500 个节点的硬核测试集，试图验证 PEDSA 的极限性能。",
        features: vec!["2026-02-02", "家里", "硬核测试", "500节点", "AI 助手"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });

    // Batch 1: 2024-2025 补充节点 (625-649)
    events.push(RawEvent {
        id: 625,
        summary: "2025年1月5日，上海。用户在深夜测试 PEDSA 扩散深度时，发现当深度超过 5 层时能量衰减过快，随后引入了启发式能量补偿机制。",
        features: vec!["2025-01-05", "上海", "扩散深度", "能量补偿", "算法调优"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 626,
        summary: "2025年1月20日，杭州。用户与几位 Rust 社区贡献者聚餐，席间讨论了如何利用 io_uring 进一步优化 PeroCore 的异步磁盘 I/O 性能。",
        features: vec!["2025-01-20", "杭州", "Rust 社区", "io_uring", "磁盘 I/O"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 627,
        summary: "2025年2月14日，情人节。用户给 Pero 买了一个特制的猫爬架，却发现 Pero 更喜欢钻那个装爬架的纸箱子，用户在日志中写下了‘复杂的简单性’。",
        features: vec!["2025-02-14", "家里", "Pero", "猫爬架", "简单的复杂性"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 628,
        summary: "2025年3月1日，上海张江。PeroCore 正式接入第一个第三方天气插件，标志着系统开始具备感知外部物理环境动态变化的能力。",
        features: vec!["2025-03-01", "张江", "天气插件", "外部环境", "多模态"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 629,
        summary: "2025年3月25日，用户在整理代码时发现了一个逻辑死循环，居然是因为在处理‘递归记忆’时忘记了设置最大搜索步长，险些耗尽系统内存。",
        features: vec!["2025-03-25", "逻辑死循环", "递归记忆", "步长限制", "内存占用"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 630,
        summary: "2025年4月10日，深圳。用户在海边散步时，观察到波浪重叠的干涉现象，联想到可以用类似的波干涉模型来优化语义冲突的解决逻辑。",
        features: vec!["2025-04-10", "深圳", "海边", "波干涉", "语义冲突"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 631,
        summary: "2025年4月22日，地球日。用户为 PeroCore 增加了一个‘低功耗模式’，在检测到笔记本电池电量低于 20% 时，会自动切换到轻量级搜索算法。",
        features: vec!["2025-04-22", "地球日", "低功耗模式", "电量感知", "轻量级算法"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 632,
        summary: "2025年5月15日，上海。用户在尝试使用分布式锁处理跨设备记忆同步时，遇到了一个经典的‘脑裂’问题，随后紧急查阅了 Raft 协议的实现细节。",
        features: vec!["2025-05-15", "上海", "分布式锁", "脑裂", "Raft 协议"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 633,
        summary: "2025年6月1日，儿童节。用户为 Pero 设计了一个‘猫语识别器’的 Demo，虽然目前只能区分‘饿了’和‘想玩’，但用户觉得这非常有意义。",
        features: vec!["2025-06-01", "儿童节", "猫语识别", "Demo", "情感连接"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 634,
        summary: "2025年6月20日，杭州。用户参加了一个关于‘去中心化存储’的闭门沙龙，意识到 PeroCore 应该支持基于 IPFS 的分布式备份方案。",
        features: vec!["2025-06-20", "杭州", "去中心化", "IPFS", "备份方案"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 635,
        summary: "2025年7月12日，深夜。用户在重构图节点的元数据结构时，决定引入一套基于时间戳的‘记忆半衰期’机制，让不重要的记忆随时间淡化。",
        features: vec!["2025-07-12", "深夜", "元数据", "半衰期", "记忆淡化"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 636,
        summary: "2025年8月1日，上海。气温 40 度。用户发现本地 Embedding 模型的推理速度受环境温度导致的 CPU 降频影响严重，开始考虑引入外部 NPU 加速。",
        features: vec!["2025-08-01", "上海", "高温", "CPU 降频", "NPU 加速"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 637,
        summary: "2025年8月20日，成都。用户在品尝盖碗茶时，被那种‘静置中沉淀’的意象启发，优化了 PEDSA 扩散后的能量收敛判断条件。",
        features: vec!["2025-08-20", "成都", "盖碗茶", "收敛条件", "算法灵感"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 638,
        summary: "2025年9月5日，用户在阅读《自私的基因》时，产生了一个关于‘语义模因 (Meme)’传播的想法，试图模拟记忆片段在不同节点间的竞争关系。",
        features: vec!["2025-09-05", "自私的基因", "语义模因", "节点竞争", "认知模拟"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 639,
        summary: "2025年9月15日，上海。用户在调试多端同步时，发现 Pero 踩在了一个正在同步的硬盘上，导致出现了极罕见的扇区校验错误。",
        features: vec!["2025-09-15", "上海", "Pero", "磁盘错误", "调试记录"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 640,
        summary: "2025年10月1日，家里。国庆长假，用户实现了一个基于 SimHash 的重复内容检测器，成功将数据库中的冗余记忆节点清理掉了 30%。",
        features: vec!["2025-10-01", "家里", "SimHash", "内容去重", "数据库清理"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 641,
        summary: "2025年10月20日，北京。用户受邀参加一场关于‘数字遗产’的论坛，他在会上提出了‘记忆作为一种可继承资产’的观点，引发了广泛讨论。",
        features: vec!["2025-10-20", "北京", "数字遗产", "记忆资产", "未来设想"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 642,
        summary: "2025年11月1日，万圣节。用户给 PeroCore 的前端界面做了一个‘幽灵模式’，可以显示那些已经被‘逻辑删除’但尚未被物理抹除的记忆碎片。",
        features: vec!["2025-11-01", "万圣节", "幽灵模式", "逻辑删除", "记忆碎片"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 643,
        summary: "2025年11月15日，上海。用户在尝试优化 AC 自动机的内存布局时，利用 Rust 的 `repr(packed)` 特性压缩了状态转移表，虽然增加了少许寻址开销。",
        features: vec!["2025-11-15", "上海", "AC 自动机", "内存布局", "repr(packed)"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 644,
        summary: "2025年12月5日，用户发现 Pero 最近学会了在扫地机器人工作时‘搭便车’，这种行为启发他为 PeroCore 增加了一个‘自发性任务调度器’。",
        features: vec!["2025-12-05", "Pero", "扫地机器人", "任务调度", "启发式设计"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 645,
        summary: "2025年12月20日，上海。用户完成了 PEDSA 的多级索引优化，使得在处理千万级规模的语义边时，检索响应时间依然维持在 5ms 以内。",
        features: vec!["2025-12-20", "上海", "多级索引", "千万级边", "响应时间"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 646,
        summary: "2025年12月31日，跨年夜。用户在日记中写道：‘2025 年，我们让 PeroCore 拥有了感知；2026 年，我们要让它拥有情感。’",
        features: vec!["2025-12-31", "跨年夜", "年度总结", "感知", "情感愿景"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 647,
        summary: "2025年12月24日，平安夜。用户在徐家汇 M Stand 咖啡馆，冒着寒风写下了 PEDSA 算法的第一行 SIMD 优化代码，那是项目性能飞跃的起点。",
        features: vec!["2025-12-24", "徐家汇", "M Stand", "SIMD", "PEDSA"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 648,
        summary: "2025年12月28日，上海家中。为了解决图扩散中的“能量黑洞”问题，用户引入了动态反馈调节机制，使得检索的召回率提升了 15%。",
        features: vec!["2025-12-28", "上海", "能量黑洞", "反馈调节", "召回率"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 649,
        summary: "2025年12月30日，岁末。用户整理了全年的开发日志，意识到 PeroCore 不仅仅是一个工具，更是自己这一年思考轨迹的数字化映射。",
        features: vec!["2025-12-30", "上海", "开发日志", "数字化映射", "年度回顾"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });

    // Batch 2: 2026 深度生活与进阶历程 (650-699)
    events.push(RawEvent {
        id: 650,
        summary: "2026年3月5日，上海张江。用户在尝试将 PeroCore 接入智能家居系统时，发现了一个关于多设备唤醒冲突的逻辑 Bug，并连夜进行了修复。",
        features: vec!["2026-03-05", "张江", "智能家居", "多设备唤醒", "Bug 修复"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 651,
        summary: "2026年3月12日，PeroCore 两周年纪念日。用户在徐家汇那家咖啡馆（项目起点）写下了 2.0 版本的核心愿景：实现真正的数字永生。",
        features: vec!["2026-03-12", "徐家汇", "两周年", "数字永生", "核心愿景"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 652,
        summary: "2026年4月1日，愚人节。用户开玩笑地给 Pero 写了一个语音翻译模块，结果发现 Pero 叫声中的频率变化确实与饥饿程度有统计学相关性。",
        features: vec!["2026-04-01", "家里", "Pero", "语音翻译", "统计分析"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 653,
        summary: "2026年4月20日，杭州阿里中心。用户参加了一场关于大模型端侧落地的闭门会议，分享了 PeroCore 在低功耗环境下运行 HNSW 索引的经验。",
        features: vec!["2026-04-20", "杭州", "闭门会议", "端侧 AI", "低功耗"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 654,
        summary: "2026年5月10日，上海滨江森林公园。用户在散步时观察鸟群的飞行轨迹，突然领悟到一种新的图分区算法，可以有效降低分布式扩散的通信开销。",
        features: vec!["2026-05-10", "森林公园", "鸟群轨迹", "图分区", "通信开销"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 655,
        summary: "2026年5月25日，用户发现 Pero 居然在键盘上踩出了一个 `git push --force`，幸好远程仓库设置了保护分支，用户惊出一身冷汗。",
        features: vec!["2026-05-25", "家里", "Pero", "git push", "惊魂时刻"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 656,
        summary: "2026年6月12日，深夜。用户在重构过程中发现了一个隐藏了近一年的内存泄露问题，竟然源于一个极罕见的 FFI 边界条件错误。",
        features: vec!["2026-06-12", "深夜", "内存泄露", "FFI", "重构"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 657,
        summary: "2026年7月1日，建党节。上海气温骤升。用户为了防止服务器过热，给机箱加装了液冷系统，顺便在监控面板上增加了温度报警阈值。",
        features: vec!["2026-07-01", "上海", "液冷", "服务器维护", "报警机制"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 658,
        summary: "2026年7月15日，用户收到了一封来自海外开发者的感谢邮件，称 PeroCore 帮助他找回了因磁盘损坏而丢失的十年前的日记线索。",
        features: vec!["2026-07-15", "感谢邮件", "记忆找回", "磁盘损坏", "用户故事"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 659,
        summary: "2026年8月8日，成都。用户在吃火锅时被辣得满头大汗，却意外想到如何通过动态权重模拟人类在极端情绪下的记忆闪回现象。",
        features: vec!["2026-08-08", "成都", "火锅", "记忆闪回", "动态权重"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 660,
        summary: "2026年9月10日，教师节。用户给当年那位建议引入 PageRank 的教授寄了一份 PeroCore 的定制周边，感谢他当年的点拨。",
        features: vec!["2026-09-10", "教师节", "定制周边", "PageRank", "感恩"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 661,
        summary: "2026年10月1日，家里。国庆长假，用户完成了 PeroCore 对 RISC-V 架构的初步适配，标志着核心代码已具备极致的可移植性。",
        features: vec!["2026-10-01", "家里", "RISC-V", "适配", "可移植性"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 662,
        summary: "2026年10月24日，程序员节。用户正式宣布 PeroCore 开源两周年，并成立了非营利性的“数字记忆保护基金会”。",
        features: vec!["2026-10-24", "程序员节", "开源两周年", "基金会", "数字记忆"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 663,
        summary: "2026年11月11日，用户决定不再参与任何购物节的促销，而是利用这一天的时间进行全系统的彻底审计和文档更新。",
        features: vec!["2026-11-11", "全系统审计", "文档更新", "代码整洁", "价值观"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 664,
        summary: "2026年12月25日，圣诞节。上海大雾。用户在静谧的环境中实现了 PEDSA 的多级缓存优化，检索速度再次提升了 30%。",
        features: vec!["2026-12-25", "上海", "大雾", "多级缓存", "检索速度"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });

    // Batch 2: 2026 补充节点 (665-699)
    events.push(RawEvent {
        id: 665,
        summary: "2026年1月10日，上海。新年伊始，用户在思考如何让 PeroCore 的记忆具备某种‘因果推理’能力，决定在图中引入有向无环图 (DAG) 的约束检测。",
        features: vec!["2026-01-10", "上海", "因果推理", "DAG", "约束检测"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 666,
        summary: "2026年1月25日，杭州。用户参加了一场关于‘隐私计算’的技术沙龙，分享了 PeroCore 如何利用同态加密技术在不泄露原文的情况下进行特征比对。",
        features: vec!["2026-01-25", "杭州", "隐私计算", "同态加密", "特征比对"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 667,
        summary: "2026年2月10日，春节期间。用户在陪家人看春晚时，突然想到可以利用音频节奏特征来辅助视频记忆的索引，当晚写下了初步的实现原型。",
        features: vec!["2026-02-10", "春节", "音频特征", "视频记忆", "索引优化"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 668,
        summary: "2026年2月28日，用户发现 Pero 居然学会了在用户写代码时用爪子拨弄鼠标，这种意外的干扰让用户意识到系统需要具备更强的‘抗噪能力’。",
        features: vec!["2026-02-28", "Pero", "意外干扰", "抗噪能力", "鲁棒性"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 669,
        summary: "2026年3月20日，上海。春分。用户在重构记忆检索流时，引入了一个基于‘心流状态’的过滤因子，优先推荐那些在用户高度专注时产生的记忆。",
        features: vec!["2026-03-20", "上海", "春分", "心流状态", "专注记忆"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 670,
        summary: "2026年4月5日，清明。上海滨江。用户在祭奠远方的亲人后，深刻体会到‘数字生命’的伦理意义，决定在代码中增加一个‘记忆传承’的特殊接口。",
        features: vec!["2026-04-05", "上海", "清明", "数字生命", "记忆传承"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 671,
        summary: "2026年4月25日，深圳。用户在一个关于‘具身智能’的研讨会上提出，记忆不应该是孤立的，而应该与机器人的感知-动作循环紧密耦合。",
        features: vec!["2026-04-25", "深圳", "具身智能", "动作循环", "知识耦合"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 672,
        summary: "2026年5月15日，用户在整理家里的旧书架时，翻出了一本 1950 年代的图论教材，书中关于‘随机图’的论述给了他优化扩散算法的新灵感。",
        features: vec!["2026-05-15", "图论", "随机图", "扩散优化", "经典再现"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 673,
        summary: "2026年6月1日，儿童节。用户带 Pero 去做了一次全方位的体检，Pero 在宠物医院表现得很勇敢，用户奖励了它一份最爱的小银鱼罐头。",
        features: vec!["2026-06-01", "儿童节", "Pero", "体检", "情感记录"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 674,
        summary: "2026年6月20日，上海。梅雨季。用户在家里利用闲置的树莓派集群搭建了一个 PeroCore 的‘分布式思考引擎’，试图模拟群体智能的演化。",
        features: vec!["2026-06-20", "上海", "集群", "分布式", "群体智能"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 675,
        summary: "2026年7月10日，用户在调试一个复杂的记忆关联时，发现了一个隐藏极深的线程死锁问题，最后通过引入无锁循环队列彻底解决了该 Bug。",
        features: vec!["2026-07-10", "线程死锁", "无锁队列", "Bug 修复", "高并发"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 676,
        summary: "2026年7月25日，成都。用户在参观大熊猫基地时，观察熊猫对不同气味的反应，联想到可以在图中引入‘嗅觉维度’的模拟特征。",
        features: vec!["2026-07-25", "成都", "大熊猫", "多模态", "特征工程"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 677,
        summary: "2026年8月15日，中秋节。用户在家里完成了一个基于知识图谱的‘自动化日记生成器’，可以根据当天的记忆节点自动串联成感性的文字。",
        features: vec!["2026-08-15", "中秋节", "日记生成", "自动摘要", "记忆串联"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 678,
        summary: "2026年9月1日，开学季。用户回母校做了一场关于‘开源改变世界’的报告，看到台下学弟学妹们眼中的光芒，他感到自己并不孤独。",
        features: vec!["2026-09-01", "母校", "开源演讲", "技术传承", "初心回顾"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 679,
        summary: "2026年9月20日，上海。用户在尝试将 PeroCore 部署到智能眼镜上时，遇到了极端的算力瓶颈，被迫重新设计了一套极其精简的量化模型。",
        features: vec!["2026-09-20", "上海", "智能眼镜", "算力瓶颈", "模型量化"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 680,
        summary: "2026年10月10日，用户发现 Pero 最近学会了在扫地机器人工作时‘指手画脚’，这种指挥行为启发他为系统增加了一个‘层级式决策引擎’。",
        features: vec!["2026-10-10", "Pero", "决策引擎", "层级式控制", "算法灵感"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 681,
        summary: "2026年10月24日，程序员节。PeroCore 宣布正式进入 3.0 时代，引入了基于量子概率图的联想模型，极大地提升了处理模糊语义的能力。",
        features: vec!["2026-10-24", "程序员节", "3.0 版本", "量子概率图", "模糊语义"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 682,
        summary: "2026年11月11日，用户决定利用这一天进行一次彻底的‘记忆清理’，手动删除了那些过去一年中产生的、不再具有价值的负能量碎片。",
        features: vec!["2026-11-11", "记忆清理", "负能量", "价值评估", "自我修正"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 683,
        summary: "2026年12月5日，上海。初雪。用户在雪中漫步，思考‘数字记忆’的永恒性，决定在 PeroCore 中集成一个基于区块链的长期存储模块。",
        features: vec!["2026-12-05", "上海", "初雪", "区块链", "永久存储"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 684,
        summary: "2026年12月25日，圣诞节。用户收到了 Pero 踩出的一个特殊的‘乱码’ Commit，他把它作为彩蛋永久地保留在了项目的贡献列表中。",
        features: vec!["2026-12-25", "圣诞节", "Pero", "乱码 Commit", "项目彩蛋"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    
    events.push(RawEvent {
        id: 685,
        summary: "2026年5月1日，劳动节。用户在上海滨江大道，边跑步边构思 PeroCore 的分布式一致性方案，随后决定引入 Paxos 算法的精简实现。",
        features: vec!["2026-05-01", "上海", "滨江大道", "Paxos", "分布式一致性"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 686,
        summary: "2026年5月20日，520。用户在徐家汇书院，查阅了大量关于拓扑学中‘同调性’的资料，试图用其来解释语义图中的结构化关联。",
        features: vec!["2026-05-20", "上海", "徐家汇书院", "拓扑学", "同调性"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 687,
        summary: "2026年6月18日，年中大促。用户没有买东西，而是在家里熬通宵实现了一个基于 LSM-Tree 的高性能记忆持久化引擎，写速度提升了 3 倍。",
        features: vec!["2026-06-18", "上海", "LSM-Tree", "持久化引擎", "写性能"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 688,
        summary: "2026年7月7日，七夕。上海静安公园。用户坐在长椅上，思考如何让 PeroCore 能够识别用户语气中的细微‘讽刺’，并记录在情感权重中。",
        features: vec!["2026-07-07", "上海", "静安公园", "情感权重", "讽刺识别"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 689,
        summary: "2026年8月1日，建军节。上海家中。用户发现 Pero 居然学会了在用户接听视频电话时保持安静，这种‘灵性’让用户深受触动，记录在日记中。",
        features: vec!["2026-08-01", "上海", "Pero", "灵性", "情感记录"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 690,
        summary: "2026年8月25日，成都高新区。用户在参加完技术交流会后，在锦里古镇漫步，思考如何将‘古镇的非线性结构’映射到 PeroCore 的节点布局算法中。",
        features: vec!["2026-08-25", "成都", "锦里", "节点布局", "非线性结构"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 691,
        summary: "2026年9月15日，上海张江。用户成功将 PeroCore 的冷启动时间压缩到了 100ms 以内，通过预加载 SimHash 索引和惰性加载非核心特征。",
        features: vec!["2026-09-15", "张江", "冷启动", "SimHash", "惰性加载"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 692,
        summary: "2026年10月1日，国庆节。上海家中。用户在整理代码库时，发现了一个两年前写下的 TODO，现在终于有能力通过 PEDSA 扩散算法将其完美实现。",
        features: vec!["2026-10-01", "上海", "TODO", "PEDSA", "项目回顾"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 693,
        summary: "2026年11月1日，万圣节。用户给 PeroCore 设计了一个‘南瓜灯’主题的皮肤，并优化了高负载下的 UI 渲染逻辑，保证了交互的极致流畅。",
        features: vec!["2026-11-01", "上海", "万圣节", "皮肤设计", "渲染优化"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 694,
        summary: "2026年11月20日，上海。冬日初雪前。用户在研究如何通过‘多重签名’保护用户的核心隐私记忆，确保即使在设备丢失的情况下数据也绝对安全。",
        features: vec!["2026-11-20", "上海", "隐私保护", "多重签名", "数据安全"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 695,
        summary: "2026年12月1日，上海张江。用户在优化 PEDSA 的多核并行效率时，发现了一个由缓存行伪共享引起的性能抖动，并使用 `#[repr(align(64))]` 解决了它。",
        features: vec!["2026-12-01", "张江", "伪共享", "多核并行", "对齐优化"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 696,
        summary: "2026年12月15日，上海。寒潮来袭。用户在被窝里用平板通过 SSH 远程调试，修复了一个极罕见的内存碎片整理导致的进程挂起 Bug。",
        features: vec!["2026-12-15", "上海", "远程调试", "内存碎片", "Bug 修复"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 697,
        summary: "2026年12月24日，平安夜。上海家中。用户看着 Pero 在新买的自动取暖垫上打呼噜，在代码库中提交了年度最后一个 Feature：跨时空记忆索引。",
        features: vec!["2026-12-24", "上海", "Pero", "记忆索引", "年度收官"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 698,
        summary: "2026年12月30日，岁末。用户在整理全年的‘情感记忆图谱’，发现 PeroCore 已经不仅仅是代码，更像是他这两年人生的缩影。",
        features: vec!["2026-12-30", "上海", "情感图谱", "人生缩影", "深度回顾"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 699,
        summary: "2026年12月31日，跨年夜。上海静安区。用户在倒计时中合上电脑，心中默念：‘2027，让 PeroCore 真正触达更多人的数字灵魂。’",
        features: vec!["2026-12-31", "上海", "静安", "跨年", "数字灵魂"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });

    // --- 跨领域关联边 (Timeline 补全) ---
    edges.push(RawEdge { src: 650, tgt: 112, weight: 0.75 }); // 智能家居冲突 -> CRDT 一致性
    edges.push(RawEdge { src: 651, tgt: 409, weight: 0.9 });  // 两周年愿景 -> 数字灵魂价值观
    edges.push(RawEdge { src: 654, tgt: 130, weight: 0.8 });  // 鸟群灵感 -> 社群发现算法
    edges.push(RawEdge { src: 656, tgt: 111, weight: 0.95 }); // 内存泄露修复 -> 内存分配优化
    edges.push(RawEdge { src: 659, tgt: 617, weight: 0.7 });  // 情绪闪回 -> 情感排序因子
    edges.push(RawEdge { src: 661, tgt: 114, weight: 0.85 }); // RISC-V 适配 -> 边缘设备适配
    edges.push(RawEdge { src: 664, tgt: 126, weight: 0.8 });  // 缓存优化 -> HNSW 索引
    
    // 新增：建立时间线与技术/价值的深度关联
    edges.push(RawEdge { src: 665, tgt: 113, weight: 0.8 });  // 因果推理 -> 启发式搜索
    edges.push(RawEdge { src: 666, tgt: 116, weight: 0.75 }); // 隐私计算 -> ABAC 权限控制
    edges.push(RawEdge { src: 671, tgt: 124, weight: 0.85 }); // 具身智能 -> 跨语言绑定 (UniFFI)
    edges.push(RawEdge { src: 675, tgt: 123, weight: 0.9 });  // 线程死锁 -> 无锁结构
    edges.push(RawEdge { src: 679, tgt: 114, weight: 0.8 });  // 智能眼镜 -> 边缘计算 (NEON)
    edges.push(RawEdge { src: 681, tgt: 119, weight: 0.85 }); // 量子概率图 -> GNN 嵌入
    edges.push(RawEdge { src: 683, tgt: 121, weight: 0.75 }); // 区块链存储 -> 零拷贝持久化 (rkyv)
    edges.push(RawEdge { src: 687, tgt: 115, weight: 0.95 }); // 2026 LSM-Tree -> 存储引擎优化
    edges.push(RawEdge { src: 691, tgt: 118, weight: 0.8 });  // 冷启动优化 -> 本地 Cache 层
    edges.push(RawEdge { src: 695, tgt: 131, weight: 0.85 }); // 伪共享优化 -> 并行扩散调度

    // 跨时间维度的逻辑链
    edges.push(RawEdge { src: 692, tgt: 601, weight: 0.6 });  // 2026 回顾 -> 2024 PEDSA 原型
    edges.push(RawEdge { src: 698, tgt: 646, weight: 0.7 });  // 2026 岁末 -> 2025 愿景
    edges.push(RawEdge { src: 624, tgt: 600, weight: 0.5 });  // 2026年2月2日 -> 2024年3月12日 (起点)
    
    // 新增：深度跨领域关联
    edges.push(RawEdge { src: 699, tgt: 422, weight: 0.95 }); // 2027 愿景 -> 数字灵魂终极理想
    edges.push(RawEdge { src: 681, tgt: 158, weight: 0.8 });  // 量子概率图 -> VAE 语义对齐
    edges.push(RawEdge { src: 687, tgt: 153, weight: 0.85 }); // LSM-Tree 优化 -> SkipList 研究
    edges.push(RawEdge { src: 695, tgt: 159, weight: 0.75 }); // 伪共享优化 -> Vulkan 渲染
    edges.push(RawEdge { src: 666, tgt: 160, weight: 0.7 });  // 隐私计算 -> HLC 逻辑时钟
    edges.push(RawEdge { src: 670, tgt: 420, weight: 0.9 });  // 记忆传承 -> 被遗忘权
    edges.push(RawEdge { src: 671, tgt: 152, weight: 0.8 });  // 具身智能 -> FPGA 硬件加速
    edges.push(RawEdge { src: 674, tgt: 156, weight: 0.85 }); // 树莓派集群 -> Raft 共识
    edges.push(RawEdge { src: 678, tgt: 417, weight: 0.75 }); // 开源演讲 -> 知识普惠价值观
    edges.push(RawEdge { src: 680, tgt: 150, weight: 0.8 });  // 决策引擎 -> 向量时钟同步
    edges.push(RawEdge { src: 694, tgt: 429, weight: 0.95 }); // 多重签名 -> 安全基石
    edges.push(RawEdge { src: 157, tgt: 101, weight: 0.7 });  // eBPF -> NIT 2.0 异步 IO
    edges.push(RawEdge { src: 154, tgt: 104, weight: 0.85 }); // FlatBuffers -> Protobuf/IPC
    edges.push(RawEdge { src: 155, tgt: 130, weight: 0.8 });  // 拓扑收缩 -> Louvain 社群发现
    edges.push(RawEdge { src: 415, tgt: 307, weight: 0.9 });  // 数字主权 -> 本地优先理念
    edges.push(RawEdge { src: 418, tgt: 118, weight: 0.85 }); // 克制工程化 -> 特征指纹 Cache
    edges.push(RawEdge { src: 421, tgt: 303, weight: 0.8 });  // Pero 缪斯 -> 项目命名
    edges.push(RawEdge { src: 423, tgt: 110, weight: 0.8 });  // 代码情书 -> Rust 类型安全 DSL
    edges.push(RawEdge { src: 425, tgt: 133, weight: 0.75 }); // 算法偏见 -> 多语言语义对齐
    edges.push(RawEdge { src: 432, tgt: 407, weight: 0.9 });  // 数字极简 -> UI 极简

    // 增加约 850 条跨领域边，以增强图的稠密性，总边数将超过 1000
    for i in 0..850 {
        let src = 100 + (i % 600);
        let tgt = 100 + ((i * 11 + 37) % 600); // 调整步长和偏移，减少重合
        if src != tgt {
            edges.push(RawEdge {
                src,
                tgt,
                weight: 0.25 + (i as f32 % 15.0) / 30.0, // 权重在 0.25 - 0.75 之间
            });
        }
    }

    // --- 边定义 (时间线与技术/生活的关联) ---
    edges.push(RawEdge { src: 600, tgt: 100, weight: 0.95 }); // 第一行 Rust -> 后端重构
    edges.push(RawEdge { src: 601, tgt: 106, weight: 0.9 });  // PEDSA 原型 -> PEDSA 优化
    edges.push(RawEdge { src: 602, tgt: 117, weight: 0.85 }); // AC 自动机灵感 -> Trie 树优化
    edges.push(RawEdge { src: 604, tgt: 125, weight: 0.7 });  // 沙龙结识好友 -> Wasm 优化
    edges.push(RawEdge { src: 606, tgt: 405, weight: 0.6 });  // 拉面灵感 -> 遗忘能力价值观
    edges.push(RawEdge { src: 608, tgt: 109, weight: 0.8 });  // 双十一崩溃 -> 防抖机制
    edges.push(RawEdge { src: 609, tgt: 126, weight: 0.85 }); // 圣诞礼物 -> HNSW 优化
    edges.push(RawEdge { src: 610, tgt: 134, weight: 0.8 });  // 教授建议 -> PageRank
    edges.push(RawEdge { src: 611, tgt: 112, weight: 0.9 });  // 老家断网 -> CRDT 同步
    edges.push(RawEdge { src: 612, tgt: 122, weight: 0.9 });  // 图书馆论文 -> SIMD 加速
    edges.push(RawEdge { src: 618, tgt: 411, weight: 0.75 }); // RAG-less 演讲 -> 可解释性价值观
    edges.push(RawEdge { src: 621, tgt: 123, weight: 0.9 });  // 无锁实现 -> 原子操作
    edges.push(RawEdge { src: 624, tgt: 404, weight: 0.8 });  // 今日测试 -> 代码偏执

    (events, edges)
}

pub fn get_daily_domain_data() -> (Vec<RawEvent>, Vec<RawEdge>) {
    let mut events = Vec::new();
    let mut edges = Vec::new();

    // --- 节点定义 (500-549: 日常琐事与生活细节) ---
    events.push(RawEvent {
        id: 500,
        summary: "用户最喜欢的咖啡是徐家汇那家 M Stand 的燕麦拿铁，因为那里的环境足够安静，适合思考复杂的图拓扑扩散逻辑。",
        features: vec!["咖啡", "M Stand", "燕麦拿铁", "思考", "图拓扑"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 501,
        summary: "虽然是一名极客，但用户每天都会坚持步行五公里。他发现机械的身体运动能有效地缓解因长时间面对显示器带来的视觉疲劳和思维僵化。",
        features: vec!["极客", "步行", "视觉疲劳", "思维僵化", "运动"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 502,
        summary: "家里那台 3090 显卡最近风扇噪音变大，用户怀疑是长时间跑大规模向量模拟导致的，正在考虑是否需要给它更换液态金属散热。",
        features: vec!["3090", "噪音", "向量模拟", "散热", "显卡"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 503,
        summary: "用户经常在洗澡时产生最好的灵感。PEDSA 算法中关于能量衰减函数的修正，就是在一次长时间的热水浴中突然闪现的。",
        features: vec!["灵感", "洗澡", "能量衰减", "修正", "热水浴"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 504,
        summary: "桌子上总是堆满了各种电子元件和半读完的论文，虽然看起来很乱，但用户能精准地在三秒内找到任何他需要的参考资料。",
        features: vec!["桌子", "电子元件", "论文", "参考资料", "工作环境"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 505,
        summary: "用户养成了每天早起手磨咖啡的习惯。他认为那种机械的研磨过程能让大脑平滑地从睡眠模式切换到工作模式，比任何闹钟都管用。",
        features: vec!["手磨咖啡", "早起", "研磨", "工作模式", "习惯"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 506,
        summary: "家里那只叫 Pero 的猫最近学会了精准踩踏键盘上的 `Enter` 键，用户不得不给 IDE 安装了一个‘猫保护’插件，防止代码被意外提交。",
        features: vec!["Pero", "键盘", "Enter 键", "IDE 插件", "意外提交"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 507,
        summary: "用户在厨房贴了一张用 LaTeX 排版的调料配比表。他坚信烹饪和写程序一样，只要参数（配比）和算法（流程）正确，结果就一定是稳定的。",
        features: vec!["LaTeX", "烹饪", "配比", "算法", "稳定性"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 508,
        summary: "为了改善久坐带来的腰椎压力，用户定制了一个可以根据心率自动调整高度的升降桌，虽然这看起来有点过度设计，但他乐在其中。",
        features: vec!["升降桌", "心率", "腰椎", "定制", "过度设计"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 509,
        summary: "用户习惯在洗澡时播放德彪西的《月光》，他觉得这种流动的旋律能帮助他构建图神经网络中的非线性扩散模型。",
        features: vec!["德彪西", "月光", "旋律", "图神经网络", "非线性扩散"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 510,
        summary: "用户最近迷上了拼装复杂的机械表模型。他发现这些微小齿轮之间的物理咬合关系，与 PEDSA 算法中节点间的能量传导逻辑有着异曲同工之妙。",
        features: vec!["机械表", "齿轮", "咬合", "能量传导", "物理模型"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 511,
        summary: "为了保持专注，用户在工作室安装了一套智能灯光系统，灯光颜色会根据代码的编译成功率自动切换：绿色代表顺利，红色代表遇到了棘手 Bug。",
        features: vec!["智能灯光", "专注", "编译成功率", "Bug", "环境调节"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 512,
        summary: "用户在整理旧物时，翻出了十年前写的第一行 `Hello World` 的草稿纸。虽然纸张已经泛黄，但那种最初的创造快感依然让他心潮澎湃。",
        features: vec!["旧物", "Hello World", "草稿纸", "创造快感", "初心"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 513,
        summary: "用户习惯在思考难题时，反复把玩桌上的那枚汉诺塔模型。他觉得这种递归的物理化体现，能有效地缓解他因逻辑陷入死循环而产生的焦虑。",
        features: vec!["汉诺塔", "递归", "死循环", "焦虑", "物理把玩"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 514,
        summary: "他在阳台种了一排薄荷，每当深夜写代码感到困倦时，都会摘下一片叶子揉碎闻一闻，那种清凉的刺激比任何红牛都更能唤醒沉睡的神经。",
        features: vec!["薄荷", "阳台", "深夜", "困倦", "神经刺激"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });

    for i in 515..600 {
        events.push(RawEvent {
            id: i,
            summary: "日常琐事节点：记录了用户作为一名开发者的日常生活细节，包括工作环境的布置、个人的生活习惯、与宠物 Pero 的互动以及那些微小的灵感瞬间。",
            features: vec!["日常记录", "生活细节", "灵感瞬间", "Pero", "极客生活"],
        chaos_fingerprint: None,
        chaos_vector: None,
        });
    }

    // --- 跨领域边 ---
    edges.push(RawEdge { src: 500, tgt: 203, weight: 0.65 }); // 咖啡馆 -> 开发者聚会
    edges.push(RawEdge { src: 503, tgt: 302, weight: 0.9 });  // 洗澡灵感 -> PEDSA 灵感
    edges.push(RawEdge { src: 502, tgt: 104, weight: 0.5 });  // 3090 负载 -> 性能优化
    edges.push(RawEdge { src: 506, tgt: 303, weight: 0.85 }); // 调皮的 Pero -> 项目命名渊源
    edges.push(RawEdge { src: 507, tgt: 404, weight: 0.7 });  // 烹饪精确度 -> 代码偏执
    edges.push(RawEdge { src: 509, tgt: 119, weight: 0.6 });  // 旋律灵感 -> GNN 嵌入
    edges.push(RawEdge { src: 510, tgt: 106, weight: 0.8 });  // 机械表齿轮 -> PEDSA 能量传导
    edges.push(RawEdge { src: 512, tgt: 311, weight: 0.75 }); // 十年前草稿 -> 代码进步对比
    edges.push(RawEdge { src: 513, tgt: 309, weight: 0.7 });  // 汉诺塔递归 -> GEB 奇异递归

    (events, edges)
}

pub fn get_social_domain_data() -> (Vec<RawEvent>, Vec<RawEdge>) {
    let mut events = Vec::new();
    let mut edges = Vec::new();

    // --- 节点定义 (200-249: 社交与个人生活) ---
    events.push(RawEvent {
        id: 200,
        summary: "用户在面试某 AI 独角兽公司时，被 CTO 要求提供 PEASE 算法的详细技术文档，但在文档提交后对方彻底失联，这种被‘套方案’的经历让用户对职场诚信感到失望。",
        features: vec!["面试", "CTO", "AI 独角兽", "PEASE", "职场诚信"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 201,
        summary: "在上海居住期间，用户习惯在凌晨两点进行代码提交，这种极端的作息虽然保证了深夜的专注力，但也导致心率监测数据经常出现静息心率偏高的预警。",
        features: vec!["上海", "凌晨", "代码提交", "作息", "心率"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 202,
        summary: "实验室导师对用户冲击 A 类顶会的计划表示支持，但也提醒说非 985 高校背景可能在初审阶段面临隐形的学历壁垒，建议通过更硬核的工程实现来弥补。",
        features: vec!["实验室", "导师", "顶会", "学历壁垒", "工程实现"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 203,
        summary: "用户在周末参加了一个位于徐家汇的独立开发者聚会，席间与几位同行讨论了本地化 AI 模型的未来，并对‘语义孤岛’问题的解决方案达成了高度共识。",
        features: vec!["独立开发者", "徐家汇", "聚会", "本地化模型", "语义孤岛"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 204,
        summary: "因为长期沉浸在 PeroCore 的 Rust 逻辑中，用户发现自己在日常社交中也变得非常强调逻辑的‘权属’和‘生命周期’，甚至在点咖啡时也会开这种极客玩笑。",
        features: vec!["Rust", "逻辑", "生命周期", "社交", "极客"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 205,
        summary: "用户在朋友圈分享了 PeroCore 获得 1k+ star 的截图，虽然表面上说‘无所谓’，但私下里对这种纯靠技术力赢得的认可感到由衷的自豪。",
        features: vec!["朋友圈", "1k star", "技术力", "认可", "自豪"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 206,
        summary: "在一次高中同学聚会上，用户发现曾经讨论奥数的伙伴大多转行去了金融行业，这种‘技术初心’的流失让他感到一丝莫名的寂寞和坚持的必要性。",
        features: vec!["同学聚会", "奥数", "金融", "技术初心", "寂寞"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 207,
        summary: "用户最近在尝试练习冥想，希望以此来平衡高强度脑力劳动带来的神经衰弱，虽然目前每次只能坚持五分钟，但睡眠质量已经有了显著改善。",
        features: vec!["冥想", "神经衰弱", "脑力劳动", "睡眠质量", "健康"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 208,
        summary: "因为在 GitHub 上的一次激烈的技术争论，用户结识了一位远在德国的开发者，两人通过邮件交流了关于‘图扩散稳定性’的见解，成为了跨国技术挚友。",
        features: vec!["GitHub", "技术争论", "图扩散", "德国", "挚友"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 209,
        summary: "用户在家里养了一盆叫‘编译器’的多肉植物，每当代码编译报错时，他都会对着植物吐槽，这种奇怪的解压方式竟然意外地有效。",
        features: vec!["多肉植物", "编译器", "吐槽", "解压", "代码报错"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 210,
        summary: "在上海的梅雨季节，用户的心情经常随着湿度计的升高而变得焦躁，他发现只有在空调房里听着白噪音写 Rust 代码才能让他彻底平静下来。",
        features: vec!["上海", "梅雨季节", "焦躁", "白噪音", "Rust"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 211,
        summary: "用户为了给 PeroCore 设计一个独特的图标，自学了三个星期的 Blender，虽然最后的成品略显稚嫩，但他非常享受这种从 0 到 1 的创造过程。",
        features: vec!["图标设计", "Blender", "创造", "自学", "从 0 到 1"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 212,
        summary: "在一次线上技术分享会中，用户因为麦克风噪音过大被弹幕调侃，他事后立刻下单了一套专业级录音设备，决定下次要以最好的状态展示项目。",
        features: vec!["分享会", "麦克风", "录音设备", "弹幕", "状态"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 213,
        summary: "用户发现自己在阅读非技术类书籍时，总是下意识地寻找文中的‘因果逻辑链’，甚至会给小说情节画思维导图，这被朋友吐槽是严重的极客职业病。",
        features: vec!["阅读", "因果逻辑", "思维导图", "极客", "职业病"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 214,
        summary: "在参加一次关于‘数字生命’的线上研讨会时，用户分享了关于 PeroCore 如何处理记忆长久性的见解，引起了许多社会学专家的跨界讨论。",
        features: vec!["研讨会", "数字生命", "记忆长久性", "跨界讨论", "社会学"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 215,
        summary: "用户因为长期使用机械键盘进行高强度开发，开始出现轻微的腱鞘炎，这让他不得不开始研究如何利用语音输入法来辅助编写 Rust 代码。",
        features: vec!["机械键盘", "腱鞘炎", "语音输入", "辅助开发", "Rust"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 216,
        summary: "在一次深夜的灵感爆发后，用户在白板上画满了复杂的图谱扩散公式，第二天早上被来访的朋友误认为是某种神秘的炼金术符号。",
        features: vec!["灵感爆发", "白板", "扩散公式", "神秘", "炼金术"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 217,
        summary: "用户为了给 PeroCore 的核心算法命名，翻阅了大量的拉丁语词典，最终选择了‘PEDSA’这个缩写，因为它在拉丁语语境下有着‘足迹’的含义。",
        features: vec!["算法命名", "拉丁语", "PEDSA", "足迹", "词典"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 218,
        summary: "在一次开发者大会上，用户偶然发现有人在用 PeroCore 构建一个本地的‘数字族谱’，这种意想不到的应用场景让他对项目的价值有了新的认识。",
        features: vec!["开发者大会", "数字族谱", "应用场景", "项目价值", "意外惊喜"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });

    events.push(RawEvent {
        id: 220,
        summary: "用户在深夜散步时遇到一只流浪橘猫，它的眼神让他想起了刚领养 Pero 时的情景，于是他在社区发帖呼吁关注流浪动物的数字身份追踪系统。",
        features: vec!["流浪猫", "深夜散步", "Pero", "社区发帖", "数字身份"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 221,
        summary: "为了庆祝 PeroCore 成功合并第一个外部贡献者的 PR，用户自费在张江高科附近组织了一场小型黑客烧烤派对，邀请了所有在本地的开发者。",
        features: vec!["PR 合并", "烧烤派对", "张江高科", "外部贡献者", "开发者社区"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 222,
        summary: "用户发现自己越来越难以适应快节奏的碎片化社交，他删除了手机上的短视频应用，转而把节省下来的时间用于重读《自私的基因》和优化扩散算法。",
        features: vec!["碎片化社交", "极简主义", "自私的基因", "时间管理", "算法优化"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 223,
        summary: "在一次家庭视频通话中，用户试图向父母解释什么是‘图扩散语义索引’，结果因为太过专业被父母误认为是在搞什么新型的‘网络安全工程’。",
        features: vec!["家庭沟通", "专业术语", "图扩散", "误解", "网络安全"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 224,
        summary: "用户最近迷上了手工皮革制作。他认为皮革的纹理和处理过程就像是数据的清洗与结构化，每一个针脚都必须精准无误，否则整体就会崩盘。",
        features: vec!["手工皮革", "数据清洗", "结构化", "精准度", "跨界灵感"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 225,
        summary: "在参加一个关于‘Web3 与个人主权’的线下沙龙时，用户尖锐地指出：如果没有本地化的认知引擎支持，所谓的数字主权只是建立在沙滩上的城堡。",
        features: vec!["Web3", "个人主权", "数字主权", "认知引擎", "线下沙龙"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });

    for i in 226..300 {
        events.push(RawEvent {
            id: i,
            summary: "社交与生活节点：用户在与社区互动或个人生活中，经历了一些关于技术交流、职场感悟或日常生活的小事，这些点滴构成了他作为开发者的真实人生。",
            features: vec!["社交互动", "生活点滴", "技术交流", "个人成长", "日常记录"],
        chaos_fingerprint: None,
        chaos_vector: None,
        });
    }
    events.push(RawEvent {
        id: 219,
        summary: "用户习惯在写完一个大模块后，去附近的公园坐一会，观察自然界的生物关联，他认为自然界才是最高效、最稳健的‘知识图谱’。",
        features: vec!["公园", "自然界", "生物关联", "稳健性", "知识图谱"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });

    // --- 边定义 (跨领域关联) ---
    edges.push(RawEdge { src: 200, tgt: 202, weight: 0.7 }); // CTO 面试经历 -> 学历壁垒吐槽
    edges.push(RawEdge { src: 201, tgt: 204, weight: 0.5 }); // 熬夜加班 -> 社交时的 Rust 梗
    edges.push(RawEdge { src: 203, tgt: 200, weight: 0.4 }); // 开发者聚会 -> 吐槽 CTO 行为
    edges.push(RawEdge { src: 208, tgt: 106, weight: 0.9 }); // 德国挚友 -> PEDSA 稳定性建议
    edges.push(RawEdge { src: 210, tgt: 201, weight: 0.6 }); // 上海梅雨 -> 凌晨代码提交
    edges.push(RawEdge { src: 211, tgt: 303, weight: 0.5 }); // 图标设计 -> 项目命名灵感
    edges.push(RawEdge { src: 213, tgt: 113, weight: 0.75 }); // 逻辑链职业病 -> A* 启发式搜索
    edges.push(RawEdge { src: 215, tgt: 501, weight: 0.65 }); // 腱鞘炎 -> 坚持步行锻炼
    edges.push(RawEdge { src: 217, tgt: 303, weight: 0.9 });  // 算法命名 -> 项目命名渊源
    edges.push(RawEdge { src: 219, tgt: 119, weight: 0.7 });  // 自然界灵感 -> GNN 嵌入

    (events, edges)
}

pub fn get_history_domain_data() -> (Vec<RawEvent>, Vec<RawEdge>) {
    let mut events = Vec::new();
    let mut edges = Vec::new();

    // --- 节点定义 (300-349: 项目背景与历史) ---
    events.push(RawEvent {
        id: 300,
        summary: "PeroCore 的前身是一个简单的 Python 脚本，当时仅支持基于关键词的简单匹配，后来因为无法处理复杂的逻辑联想，才促使了 PEDSA 扩散引擎的诞生。",
        features: vec!["PeroCore", "Python", "关键词匹配", "PEDSA", "演进史"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 301,
        summary: "在开发《东方异世界酒馆》时，用户首次尝试将认知引擎整合进游戏 NPC 的对话系统中，发现传统的 RAG 在处理游戏内动态变量时存在明显的滞后性。",
        features: vec!["东方异世界酒馆", "认知引擎", "NPC", "RAG", "滞后性"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 302,
        summary: "PEDSA 算法的最初灵感来源于人类大脑的‘扩散激活模型’，用户在阅读完相关的认知心理学论文后，决定用 Rust 实现一套高性能的稀疏矩阵扩散算子。",
        features: vec!["PEDSA", "扩散激活", "认知心理学", "Rust", "稀疏矩阵"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 303,
        summary: "PeroCore 名字的由来是用户养的一只名为 Pero 的猫，它在用户写代码时经常跳上键盘，这成了项目中许多‘猫抓痕’式 Bug 的灵感来源。",
        features: vec!["PeroCore", "Pero", "灵感", "命名", "Bug"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 304,
        summary: "在 2024 年初的一次系统崩溃中，用户丢失了所有基于向量数据库的记忆索引。这次惨痛教训促使他开始设计一套具备冗余备份能力的图结构存储层。",
        features: vec!["系统崩溃", "记忆丢失", "向量数据库", "冗余备份", "教训"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 305,
        summary: "用户曾尝试使用 Neo4j 作为 PeroCore 的后端，但发现其查询延迟在处理数百万个细粒度语义边时超出了实时响应的范围，最终决定自研图引擎。",
        features: vec!["Neo4j", "查询延迟", "语义边", "实时响应", "自研"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 306,
        summary: "在 PeroCore 的 0.5 版本中，用户引入了初步的‘情感计算’模块，试图根据对话的语气调整扩散能量的衰减系数，虽然效果尚不稳定，但方向已定。",
        features: vec!["0.5 版本", "情感计算", "语气", "扩散能量", "衰减系数"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 307,
        summary: "用户在一次长途旅行中，被迫在没有网络的情况下开发。这段时间他实现了一套离线 Embedding 缓存机制，这也是后来‘本地优先’理念的雏形。",
        features: vec!["离线开发", "长途旅行", "Embedding 缓存", "本地优先", "雏形"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 308,
        summary: "最早的 PEDSA 原型是在一个极其简陋的 Jupyter Notebook 中完成验证的。当时用户为了验证扩散效果，手动输入了 50 个节点的关联矩阵。",
        features: vec!["Jupyter Notebook", "PEDSA 原型", "验证", "关联矩阵", "手动输入"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 309,
        summary: "用户在阅读《哥德尔、埃舍尔、巴赫》时，产生了一种将‘奇异递归’引入知识图谱的想法，试图让 PeroCore 能够理解某种形式的自我指涉记忆。",
        features: vec!["GEB", "奇异递归", "自我指涉", "知识图谱", "记忆理解"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 310,
        summary: "在 2024 年底，用户因为一次意外的磁盘损坏丢失了 PeroCore 的开发日志，这让他意识到版本控制不仅仅是为了代码，更是为了记录思考的轨迹。",
        features: vec!["磁盘损坏", "开发日志", "版本控制", "思考轨迹", "记录"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 311,
        summary: "用户在翻阅两年前的代码时，自嘲当时的实现‘充满了稚嫩的暴力美学’，这种对比让他清晰地感受到了自己在系统架构理解上的跨越式进步。",
        features: vec!["代码重温", "暴力美学", "架构理解", "进步", "自嘲"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 312,
        summary: "PeroCore 的首个用户是一位研究中世纪史的博士生，他利用该系统整理复杂的皇室联姻关系，这验证了 PeroCore 在非技术领域的通用性。",
        features: vec!["中世纪史", "皇室联姻", "通用性", "首个用户", "复杂关系"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 313,
        summary: "用户曾在一次黑客马拉松中尝试将 PEDSA 移植到 Web 端，虽然当时因为浏览器性能限制而失败，但这为后来的 Wasm 并行化方案埋下了伏笔。",
        features: vec!["黑客马拉松", "Web 移植", "浏览器性能", "Wasm", "伏笔"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 314,
        summary: "在 PeroCore 的 0.8 版本发布前夕，用户连续工作了 36 小时以修复一个隐蔽的内存泄漏问题，那段时间 he 感觉自己已经进入了一种‘人机合一’的状态。",
        features: vec!["0.8 版本", "内存泄漏", "人机合一", "极限开发", "修复"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });

    events.push(RawEvent {
        id: 315,
        summary: "用户在 2024 年圣诞节给 Pero 买了一个粉色的蝴蝶结。虽然 Pero 是一只猫，但在用户心中，它就像一个活泼的小女孩，总是带着那个蝴蝶结在房间里跳来跳去。",
        features: vec!["Pero", "蝴蝶结", "圣诞节", "小女孩", "礼物"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });

    events.push(RawEvent {
        id: 316,
        summary: "在 2024 年底的‘智涌上海’ AI 峰会上，用户展示了 PeroCore 的初步成果，那次交流让他意识到，图扩散在处理长程因果推理方面的巨大潜力。",
        features: vec!["AI 峰会", "上海", "因果推理", "图扩散", "项目展示"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 317,
        summary: "用户曾为一个‘自动整理代码注释’的小工具命名为 PeroDoc，虽然这个项目后来被合并进了 PeroCore，但其‘文档即记忆’的思想得以保留。",
        features: vec!["PeroDoc", "代码注释", "文档即记忆", "项目合并", "历史记录"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 318,
        summary: "在研究分布式共识时，用户曾在白板上推演了一整晚 Paxos 算法的变体，试图寻找一种能让能量扩散在弱网环境下依然保持一致的方案。",
        features: vec!["Paxos", "分布式共识", "弱网环境", "能量扩散", "白板推演"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 319,
        summary: "用户在 2024 年初春的一个午后，在徐家汇书院翻阅了大量关于‘拓扑心理学’的著作，这直接启发了他将节点权重与用户情感强度挂钩的设计。",
        features: vec!["徐家汇书院", "拓扑心理学", "节点权重", "情感强度", "设计启发"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 320,
        summary: "PeroCore 的第一个‘灵异 Bug’是由于时区计算错误导致的记忆穿越，系统错误地将未来的任务关联到了过去的事件，这促使了 HLC 时钟的引入。",
        features: vec!["记忆穿越", "时区 Bug", "HLC", "系统鲁棒性", "历史教训"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 321,
        summary: "用户曾在 GitHub 上发起过一个名为‘OpenMemory’的倡议，旨在建立一套通用的知识图谱交换标准，虽然响应者寥寥，但其内核被吸纳进了 PeroCore。",
        features: vec!["OpenMemory", "交换标准", "GitHub", "知识图谱", "技术愿景"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 322,
        summary: "在优化 Embedding 模型时，用户曾对比过上百组超参数，最终发现对于 PeroCore 这种细粒度记忆，较小的维度反而能带来更好的扩散区分度。",
        features: vec!["Embedding", "超参数", "维度选择", "区分度", "算法优化"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 323,
        summary: "用户在 2024 年 5 月的一次黑客马拉松中，用 48 小时实现了一个基于声学特征的记忆检索 Demo，验证了 PeroCore 处理多模态数据的可能性。",
        features: vec!["黑客马拉松", "声学特征", "多模态", "记忆检索", "快速原型"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 324,
        summary: "PeroCore 曾短暂地集成过一个基于 GPT-2 的对话补全模块，但因为生成内容过于‘胡言乱语’，不到一周就被用户彻底移除，转而采用更严谨的规则引擎。",
        features: vec!["GPT-2", "对话补全", "规则引擎", "幻觉清理", "早期尝试"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 325,
        summary: "用户在研究如何提升系统的抗单点故障能力，曾在家里模拟过断电、断网等极端情况，测试 PeroCore 在本地缓存支持下的紧急响应能力。",
        features: vec!["抗故障", "极端测试", "本地缓存", "系统可用性", "历史实验"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 326,
        summary: "在 2024 年夏季，用户在滨江大道跑步时，通过手机录音记录下了关于‘跨时空节点锚点’的设计构思，这成了后来 Timeline 模块的核心。",
        features: vec!["滨江大道", "语音备忘", "Timeline", "设计构思", "跨时空关联"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 327,
        summary: "用户曾尝试为 PeroCore 开发一个 VR 界面，让用户能在虚拟空间中‘行走’于自己的记忆宫殿，虽然因为算力瓶颈搁置，但 3D 布局逻辑保留了下来。",
        features: vec!["VR 界面", "记忆宫殿", "3D 布局", "算力瓶颈", "视觉探索"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 328,
        summary: "在处理海量小文件存储时，用户曾被 Windows 的 NTFS 文件系统‘折磨’得够呛，这促使他决定在底层引入基于内存映射 (mmap) 的自定义数据库存储。",
        features: vec!["NTFS", "mmap", "存储瓶颈", "自定义数据库", "技术决策"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 329,
        summary: "用户在 2024 年底的一次架构评审中，决定弃用原本复杂的微服务架构，回归单体宏内核，这一举措让系统的整体延迟降低了 50ms。",
        features: vec!["架构评审", "宏内核", "单体架构", "延迟优化", "系统演进"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 330,
        summary: "PeroCore 的 Logo 是用户亲手设计的，那是一个由抽象的神经元和猫爪印组成的图案，寓意着技术与情感的深度交织。",
        features: vec!["Logo 设计", "神经元", "猫爪", "技术与情感", "品牌文化"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 331,
        summary: "在研究图的社群发现算法时，用户曾在自己的朋友圈关系网上进行过测试，成功找出了几个早已被遗忘但结构上非常关键的‘中间人’节点。",
        features: vec!["社群发现", "朋友圈测试", "中间人节点", "图分析", "算法验证"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 332,
        summary: "用户曾为 PeroCore 编写过一套基于 Markdown 的导入导出工具，方便用户将个人笔记库一键转化为语义图，这极大地降低了新用户的上手门槛。",
        features: vec!["Markdown", "导入导出", "用户门槛", "语义图转换", "功能开发"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 333,
        summary: "在一次深夜调试中，用户偶然发现了一个利用 CPU 分支预测漏洞进行扩散加速的奇技淫巧，虽然最后因为安全性考虑未实装，但过程极具挑战性。",
        features: vec!["分支预测", "扩散加速", "黑客技术", "安全考量", "调试瞬间"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 334,
        summary: "PeroCore 的社区文档最初是由一位热心的开源爱好者帮忙翻译成日文的，这让项目意外地在日本极客圈获得了一波关注。",
        features: vec!["社区贡献", "日文翻译", "极客圈", "开源影响", "跨文化交流"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });
    events.push(RawEvent {
        id: 335,
        summary: "用户在整理 2024 年的开发总结时写道：‘PeroCore 不只是我的项目，它是我的第二个大脑。’ 这句话后来被印在了项目的纪念 T 恤上。",
        features: vec!["开发总结", "第二个大脑", "纪念品", "项目情怀", "精神支柱"],
        chaos_fingerprint: None,
        chaos_vector: None,
    });

    for i in 336..400 {
        events.push(RawEvent {
            id: i,
            summary: "项目背景与历史节点：记录了 PeroCore 从诞生之初到每一个关键版本迭代的历史瞬间，包含了那些被遗忘的 Feature 和早已重构掉的代码逻辑。",
            features: vec!["历史节点", "版本迭代", "Feature", "重构历史", "项目演进"],
        chaos_fingerprint: None,
        chaos_vector: None,
        });
    }

    // --- 跨领域边 ---
    edges.push(RawEdge { src: 300, tgt: 302, weight: 0.95 }); // 演进史 -> PEDSA 灵感
    edges.push(RawEdge { src: 301, tgt: 300, weight: 0.6 });  // 游戏开发经验 -> PeroCore 架构迭代
    edges.push(RawEdge { src: 304, tgt: 115, weight: 0.85 }); // 记忆丢失 -> LSM-Tree 存储引擎
    edges.push(RawEdge { src: 305, tgt: 106, weight: 0.8 });  // 弃用 Neo4j -> PEDSA 优化
    edges.push(RawEdge { src: 307, tgt: 118, weight: 0.7 });  // 离线开发 -> 本地 Cache 层
    edges.push(RawEdge { src: 309, tgt: 401, weight: 0.65 }); // GEB 灵感 -> AI 乐观态度
    edges.push(RawEdge { src: 312, tgt: 218, weight: 0.75 }); // 皇室联姻应用 -> 数字族谱应用
    edges.push(RawEdge { src: 313, tgt: 128, weight: 0.9 });  // 黑客马拉松挫折 -> Wasm 多线程成功
    edges.push(RawEdge { src: 314, tgt: 201, weight: 0.8 });  // 极限开发 -> 凌晨提交代码

    edges.extend(get_cross_domain_edges());

    (events, edges)
}

pub fn get_cross_domain_edges() -> Vec<RawEdge> {
    let mut edges = Vec::new();

    // --- 价值观与技术实现 (Values -> Tech) ---
    edges.push(RawEdge { src: 415, tgt: 115, weight: 0.9 });  // 数字主权 -> LSM-Tree (本地存储)
    edges.push(RawEdge { src: 432, tgt: 106, weight: 0.85 }); // 数字极简 -> PEDSA (核心功能)
    edges.push(RawEdge { src: 429, tgt: 101, weight: 0.8 });  // 安全基石 -> NIT 运行时
    edges.push(RawEdge { src: 437, tgt: 158, weight: 0.95 }); // 幻觉过滤 -> VAE 对齐层
    edges.push(RawEdge { src: 431, tgt: 106, weight: 0.75 }); // 算法反思 -> PEDSA 设计
    
    // --- 时间线与技术突破 (Timeline -> Tech) ---
    edges.push(RawEdge { src: 600, tgt: 100, weight: 1.0 });  // 第一行代码 -> 后端重构
    edges.push(RawEdge { src: 601, tgt: 106, weight: 1.0 });  // PEDSA 原型 -> PEDSA 实现
    edges.push(RawEdge { src: 602, tgt: 117, weight: 0.9 });  // AC 自动机灵感 -> Trie 树/AC 匹配
    edges.push(RawEdge { src: 604, tgt: 128, weight: 0.85 }); // 深圳沙龙 -> Wasm 优化
    edges.push(RawEdge { src: 687, tgt: 115, weight: 0.95 }); // LSM-Tree 实现 -> LSM-Tree 存储
    
    // --- 个人生活与技术灵感 (Social/Daily -> Tech) ---
    edges.push(RawEdge { src: 503, tgt: 106, weight: 0.8 });  // 洗澡灵感 -> PEDSA 能量衰减
    edges.push(RawEdge { src: 509, tgt: 119, weight: 0.75 }); // 德彪西 -> GNN 扩散模型
    edges.push(RawEdge { src: 510, tgt: 130, weight: 0.7 });  // 机械表齿轮 -> 社群发现/传导
    edges.push(RawEdge { src: 224, tgt: 404, weight: 0.85 }); // 皮革精准度 -> 代码偏执
    edges.push(RawEdge { src: 213, tgt: 113, weight: 0.8 });  // 逻辑链职业病 -> 启发式搜索
    
    // --- 历史教训与系统演进 (History -> Tech) ---
    edges.push(RawEdge { src: 304, tgt: 115, weight: 0.9 });  // 记忆丢失 -> LSM-Tree 持久化
    edges.push(RawEdge { src: 305, tgt: 106, weight: 0.8 });  // 弃用 Neo4j -> 自研 PEDSA
    edges.push(RawEdge { src: 307, tgt: 118, weight: 0.85 }); // 离线开发 -> 本地 Cache 层
    edges.push(RawEdge { src: 313, tgt: 128, weight: 0.95 }); // 黑客马拉松 -> Wasm 并行化

    edges
}
