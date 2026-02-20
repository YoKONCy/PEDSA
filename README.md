<div align="center">

<!-- 动态渐变头图 -->
<img src="https://capsule-render.vercel.app/api?type=waving&color=0:FFB6C1,50:FF69B4,100:FF1493&height=220&section=header&text=PEDSA&fontSize=70&fontColor=ffffff&fontAlignY=35&desc=Brain-inspired%20RAG-less%20Memory%20Engine&descSize=25&descAlignY=60&animation=twinkling&fontAlign=50" width="100%"/>

<br/>

<!-- 动态打字效果 Slogan -->
<a href="https://github.com/YoKONCy/PEDSA">
  <img src="https://readme-typing-svg.demolab.com?font=Noto+Serif+SC&weight=600&size=28&duration=4000&pause=1000&color=FF1493&center=true&vCenter=true&multiline=true&repeat=false&width=800&height=70&lines=Beyond+Vectors,+Towards+Cognition;%E8%B6%85%E8%B6%8A%E5%90%91%E9%87%8F%EF%BC%8C%E8%BF%88%E5%90%91%E8%AE%A4%E7%9F%A5;Graph+Diffusion+%E2%9C%A6+Chaos+Theory+%E2%9C%A6+Cognitive+Computing" alt="Slogan" />
</a>

<br/>

<!-- 声明 -->
<a href="./1. 项目白皮书.md">
  <img src="https://img.shields.io/badge/Status-Graph_Diffusion_Engineering_Implementation-FF1493?style=for-the-badge&logo=rust" alt="Status">
</a>

<br/><br/>

<!-- 徽章导航 -->
<a href="#-technical-architecture">
  <img src="https://img.shields.io/badge/Architecture-Rust_Core-orange?style=for-the-badge&logo=rust&logoColor=white" alt="Arch">
</a>
&nbsp;
<a href="#-theoretical-foundation">
  <img src="https://img.shields.io/badge/Theory-Cognitive_Science-ff69b4?style=for-the-badge&logo=brain&logoColor=white" alt="Theory">
</a>
&nbsp;
<a href="#-performance">
  <img src="https://img.shields.io/badge/Performance-10M_Nodes_@_132ms-success?style=for-the-badge&logo=speedtest&logoColor=white" alt="Perf">
</a>
&nbsp;
<a href="#-features">
  <img src="https://img.shields.io/badge/Feature-RAG--less-blue?style=for-the-badge&logo=graph&logoColor=white" alt="Feature">
</a>

<br/><br/>

</div>

---

<br/><br/>

<div align="center">

> **"如无必要，勿增实体；少即是好，易即是难"**
>
> *"Entities should not be multiplied unnecessarily."*

<br/>

<img src="./github_src/cover.png" width="100%" alt="PEDSA Cover"/>

<br/>

</div>

<div align="center">

<!-- 动态分隔线 -->
<img src="https://user-images.githubusercontent.com/73097560/115834477-dbab4500-a447-11eb-908a-139a6edaec5c.gif" width="100%">

<br/>

## 📋 Table of Contents

<details open>
<summary><b>Quick Navigation</b></summary>
<br/>

| Section | Description                                 |                Link                 |
| :-----: | :------------------------------------------ | :---------------------------------: |
|   📖    | **Whitepaper** - 项目核心白皮书             |    [Read](./1.%20项目白皮书.md)     |
|   🧠    | **Philosophy** - 设计哲学与核心理念         |        [Jump](#-philosophy)         |
|   🔬    | **Theory** - 脑科学与认知科学理论基础       |  [Jump](#-theoretical-foundation)   |
|   🏗️    | **Architecture** - V3 架构演进与技术细节    |  [Jump](#-technical-architecture)   |
|   🎮    | **Usage Guide** - 双 LLM 管道最佳实践       |       [Jump](#-usage-guide)         |
|    ⚡    | **Performance** - 千万级节点压测报告        |        [Jump](#-performance)        |
|   🔮    | **Dual-Track** - 理性与混沌双轨检索机制     | [Jump](#-dual-track-retrieval-flow) |

</details>

<br/>

<div align="center">
  <img src="https://count.getloli.com/get/@PEDSA?theme=rule34" alt="PEDSA Count" />
</div>

<br/>

<!-- 动态分隔线 -->
<img src="https://user-images.githubusercontent.com/73097560/115834477-dbab4500-a447-11eb-908a-139a6edaec5c.gif" width="100%">

<br/>
</div>

## 🌟 Philosophy

**PEDSA** (Parallel Energy Diffusion Spreading Activation) 是一个基于脑科学与认知心理学理论构建的 **RAG-less** 记忆检索引擎。

在当前 RAG 检索为主流的时代，大多数系统依赖于单纯的向量相似度检索。然而，人类的记忆并非简单的“关键词匹配”，而是基于**联想**、**情感**与**时空**的复杂网络。


## 🔬 Theoretical Foundation

PEDSA 的底层架构深度借鉴了现有的类脑记忆提取与情感认知的一系列成熟理论：

1.  **激活扩散模型 (Spreading Activation Model)**
    *   记忆以语义网络形式存储，能量沿关联路径扩散，实现“由点及面”的联想检索。
2.  **艾宾浩斯遗忘曲线 (Ebbinghaus Forgetting Curve)**
    *   引入 `Ebbinghaus Decay` 与 `tau` 半衰期，动态计算记忆节点的残余能量，模拟人类对近期事件的鲜活记忆。
3.  **普拉切克情感轮 (Plutchik's Wheel of Emotions)**
    *   在 SimHash 指纹中预留 `MASK_AFFECTIVE`，通过位运算模拟情感色彩的共鸣 (Affective Resonance)。
4.  **混沌情感理论 (Chaos Theory of Emotion)**
    *   引入 `Chaotic Track`，当理性检索无法给出确定答案时，通过微小的扰动触发生维度的“灵感”跃迁。
5.  **侧向抑制与反向抑制 (Lateral & Inverse Inhibition)**
    *   模拟神经系统的信号处理，抑制弱噪声信号（侧向抑制）并降低高频通用概念的权重（反向抑制）。

<br/>

## 🏗️ Technical Architecture

PEDSA V3 是一次彻底的底层重构，从传统的“内存对象树”转向了“面向硬件优化”的工业级架构。

### Core Technologies

- **Index-Carrier Split (SoA)**: 将指纹、ID 和元数据提取为 **Struct of Arrays** 布局，极大提升缓存命中率。
- **Zero-copy mmap**: 利用内存映射技术，实现 10M+ 节点的**微秒级冷启动**。
- **SIMD Acceleration**: 使用 **AVX2** 指令集并行处理 SimHash 距离计算，性能提升显著。
- **LSM-tree Buffer**: 引入热插入缓冲区，支持无停机增量更新。
- **Temporal Intelligence**: 智能解析“今天”、“昨天”、“刚才”等相对时间概念，并将其映射到绝对时间轴。

### 🔄 Dual-Track Retrieval Flow

当用户输入文本时，PEDSA 采用**理性 (Rational) 与 混沌 (Chaotic) 双轨并行架构**：

```mermaid
flowchart TD
    Input([User Input]) --> FeatureExtract[Feature Extraction & AC Automaton]
    
    subgraph "Rational Track (Graph Diffusion)"
        FeatureExtract --> Ontology[Ontology Activation]
        Ontology --> Diffusion[Graph Energy Diffusion]
        Diffusion --> Lateral[Lateral Inhibition (Top-K)]
        Diffusion --> Inverse[Inverse Inhibition (Degree Penalty)]
        Lateral --> Candidates1[Rational Candidates]
    end
    
    subgraph "Chaotic Track (Vector Resonance)"
        FeatureExtract --> L1Scan[L1: 1-bit Chaos Fingerprint Scan]
        L1Scan --> L2Scan[L2: f16 Vector Cosine Similarity]
        L2Scan --> Threshold{Similarity > 0.95?}
        Threshold -- Yes --> Candidates2[Chaotic Candidates]
        Threshold -- No --> Discard[Discard]
    end
    
    Candidates1 --> Fusion[Weighted Fusion]
    Candidates2 --> Fusion
    
    Fusion --> Decay[Ebbinghaus Decay]
    Decay --> ReRank[Temporal & Affective Re-ranking]
    ReRank --> Output([Final Memory Recall])
```

<br/>

## 🎮 Usage Guide

为了让 PEDSA 发挥出“越用越聪明”的记忆特性，我们推荐采用 **"Dual-LLM Pipeline" (双 LLM 管道)** 的最佳实践模式。

### 1. The Builder (构建者)
**Role**: 负责将非结构化的对话流转化为结构化的图谱指令。

*   **Trigger**: 在每轮对话结束 (Post-Chat) 后调用。
*   **Prompt**: 使用 [2. 图谱构建提示词.md](./2.%20图谱构建提示词.md)。
*   **Input**: 当前对话上下文 (User + AI) + Reference Time。
*   **Output**: 生成 `new_event` (事件节点) 与 `ontology_updates` (本体关联) 的增量 JSON。
*   **Effect**: 随着对话轮次的增加，Ontology 层会自动捕获用户偏好、习惯与专有词汇，使检索精度呈指数级提升。

### 2. The Arbiter (仲裁者)
**Role**: 负责解决逻辑冲突与知识一致性维护。

*   **Trigger**: 仅当 Builder 返回 `action: "replace"` 指令时触发 (例如：用户说 "我不喜欢红色了，现在喜欢蓝色")。
*   **Prompt**: 使用 [3. 图谱修复提示词.md](./3.%20图谱修复提示词.md)。
*   **Mechanism**:
    1.  PEDSA 引擎提取冲突主体的 **1-hop Ego-network** (局部子图)。
    2.  Arbiter LLM 接收子图与新事实，判断哪些旧边 (Old Edges) 需要被物理删除。
    3.  引擎执行 `delete_targets` 清理操作，实现知识自愈。

> **Pro Tip**: 这种 "Write-heavy" 的设计确保了 PEDSA 不需要昂贵的全局重训练，每一次对话都是一次实时的微调 (Real-time Fine-tuning)。

<br/>

## ⚡ Performance

针对 **1000 万级** 节点的实时检索场景评估：

| Metric | Value | Note |
| :--- | :--- | :--- |
| **Total Nodes** | **10,000,000+** | 包含磁盘数据与热插入缓冲区 |
| **Memory Usage** | **~380 MB** | 极度优化的 SoA 存储结构 |
| **Scan Latency** | **132 ms** | 全量混沌检索扫描 (SIMD) |
| **Throughput** | **High** | 支持高并发查询 |
| **Hot Insert** | **Supported** | 无需重启即可实现增量更新 |
| **Precision@k** | **100%** | Top-1 & Top-5 (基于 15 个硬核 Ground Truth 测试) |

<br/>

## 🛠️ Project Structure

```bash
PEDSA/
├── models/             # 预训练模型 (BGE, etc.)
├── src/
│   ├── main.rs         # 入口文件
│   ├── lib.rs          # 库定义
│   ├── embedding.rs    # 向量嵌入与模型加载
│   ├── inference_engine.rs # 核心推理引擎
│   ├── storage.rs      # 存储引擎 (mmap, SoA)
│   ├── dataset.rs      # 数据集生成与处理
│   └── ...
├── Cargo.toml          # Rust 项目配置
├── 1. 项目白皮书.md      # 详细技术文档
├── 2. 图谱构建提示词.md    # 核心 Prompts - 构建
├── 3. 图谱修复提示词.md    # 核心 Prompts - 仲裁
└── README.md           # 你正在读的文档
```

<br/>

## 📝 Note

> "这个项目本质上是一个 Graph 引擎。具体能检索到什么，完全取决于图构建得怎么样，所以几乎没什么通用性。不过，就我自用而言，单论在 AIRP 场景的记忆检索下，效果还挺不错的就是了。"

<br/>

<div align="center">
  <img src="https://capsule-render.vercel.app/api?type=waving&color=0:FFB6C1,50:FF69B4,100:FF1493&height=80&section=footer" width="100%"/>
</div>
