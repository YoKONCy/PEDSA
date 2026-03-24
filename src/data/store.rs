#![allow(dead_code)]

use half::f16;
use crate::data::storage::ChaosFingerprint;

// ============================================================================
// 存储后端 Trait 定义
// ============================================================================

/// 图类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphType {
    /// 记忆图 (Memory Graph)
    Memory,
    /// 本体定义图 (Ontology Graph)
    Ontology,
}

/// 持久化节点 (扁平化，不依赖 SmallVec 等内部类型)
#[derive(Debug, Clone)]
pub struct StoredNode {
    pub id: i64,
    pub node_type: u8,         // 0=Feature, 1=Event
    pub content: String,
    pub fingerprint: u64,
    pub timestamp: u64,
    pub emotions: Vec<u8>,
    pub prev_event: Option<i64>,
    pub next_event: Option<i64>,
}

/// 持久化图边
#[derive(Debug, Clone)]
pub struct StoredEdge {
    pub src: i64,
    pub tgt: i64,
    pub strength: u16,
    pub edge_type: u8,
}

/// 持久化向量记录
#[derive(Debug, Clone)]
pub struct StoredVector {
    pub id: i64,
    pub fingerprint: ChaosFingerprint,
    pub vector: Vec<f16>,
}

/// 持久化关键词记录
#[derive(Debug, Clone)]
pub struct StoredKeyword {
    pub keyword: String,
    pub node_id: i64,
}

/// PEDSA 存储后端 trait
///
/// 所有方法使用扁平化的 Stored* 类型，与 AdvancedEngine 内部类型解耦。
/// AdvancedEngine 本身不实现此 trait —— 它保持原有内存架构不变。
/// 此 trait 用于引擎之外的数据持久化/同步层。
pub trait PedsaStore {
    type Error: std::fmt::Debug + std::fmt::Display;

    // ── 节点 ──
    fn save_node(&mut self, node: &StoredNode) -> Result<(), Self::Error>;
    fn save_nodes_batch(&mut self, nodes: &[StoredNode]) -> Result<(), Self::Error>;
    fn load_all_nodes(&self) -> Result<Vec<StoredNode>, Self::Error>;
    fn node_count(&self) -> Result<usize, Self::Error>;

    // ── 图边 ──
    fn save_edge(&mut self, graph_type: GraphType, edge: &StoredEdge) -> Result<(), Self::Error>;
    fn save_edges_batch(&mut self, graph_type: GraphType, edges: &[StoredEdge]) -> Result<(), Self::Error>;
    fn load_all_edges(&self, graph_type: GraphType) -> Result<Vec<StoredEdge>, Self::Error>;
    fn delete_edges(&mut self, graph_type: GraphType, src: i64, targets: &[i64]) -> Result<usize, Self::Error>;

    // ── 关键词索引 ──
    fn save_keyword(&mut self, kw: &StoredKeyword) -> Result<(), Self::Error>;
    fn save_keywords_batch(&mut self, kws: &[StoredKeyword]) -> Result<(), Self::Error>;
    fn load_all_keywords(&self) -> Result<Vec<StoredKeyword>, Self::Error>;

    // ── 向量存储 ──
    fn save_vector(&mut self, vec: &StoredVector) -> Result<(), Self::Error>;
    fn save_vectors_batch(&mut self, vecs: &[StoredVector]) -> Result<(), Self::Error>;
    fn load_all_vectors(&self) -> Result<Vec<StoredVector>, Self::Error>;

    // ── 生命周期 ──
    fn flush(&mut self) -> Result<(), Self::Error>;
    fn clear(&mut self) -> Result<(), Self::Error>;
}

// ============================================================================
// 引擎同步工具函数
// ============================================================================

use crate::core::engine::AdvancedEngine;
use crate::core::types::*;
use smallvec::SmallVec;

/// 从 AdvancedEngine 导出全部数据到任意 PedsaStore
pub fn export_engine_to_store<S: PedsaStore>(engine: &AdvancedEngine, store: &mut S) -> Result<(), S::Error> {
    // 1. 导出节点
    let nodes: Vec<StoredNode> = engine.nodes.values().map(|n| StoredNode {
        id: n.id,
        node_type: match n.node_type { NodeType::Feature => 0, NodeType::Event => 1 },
        content: n.content.clone(),
        fingerprint: n.fingerprint,
        timestamp: n.timestamp,
        emotions: n.emotions.to_vec(),
        prev_event: n.prev_event,
        next_event: n.next_event,
    }).collect();
    store.save_nodes_batch(&nodes)?;

    // 2. 导出记忆图边
    let mut memory_edges = Vec::new();
    for (&src, edges) in &engine.graph {
        for e in edges {
            memory_edges.push(StoredEdge {
                src,
                tgt: e.target_node_id,
                strength: e.connection_strength,
                edge_type: e.edge_type,
            });
        }
    }
    store.save_edges_batch(GraphType::Memory, &memory_edges)?;

    // 3. 导出本体图边
    let mut ontology_edges = Vec::new();
    for (&src, edges) in &engine.ontology_graph {
        for e in edges {
            ontology_edges.push(StoredEdge {
                src,
                tgt: e.target_node_id,
                strength: e.connection_strength,
                edge_type: e.edge_type,
            });
        }
    }
    store.save_edges_batch(GraphType::Ontology, &ontology_edges)?;

    // 4. 导出关键词
    let kws: Vec<StoredKeyword> = engine.keyword_to_node.iter().map(|(k, &v)| StoredKeyword {
        keyword: k.clone(),
        node_id: v,
    }).collect();
    store.save_keywords_batch(&kws)?;

    // 5. 导出向量
    let mut vecs = Vec::new();
    for (i, &id) in engine.chaos_store.ids.iter().enumerate() {
        vecs.push(StoredVector {
            id,
            fingerprint: engine.chaos_store.fingerprints[i],
            vector: engine.chaos_store.vectors[i].clone(),
        });
    }
    store.save_vectors_batch(&vecs)?;

    store.flush()?;
    Ok(())
}

/// 从任意 PedsaStore 加载数据到 AdvancedEngine
pub fn import_store_to_engine<S: PedsaStore>(store: &S, engine: &mut AdvancedEngine) -> Result<(), S::Error> {
    // 1. 加载节点
    for sn in store.load_all_nodes()? {
        let node_type = if sn.node_type == 0 { NodeType::Feature } else { NodeType::Event };
        let mut emotions = SmallVec::new();
        for &e in &sn.emotions {
            emotions.push(e);
        }
        let node = Node {
            id: sn.id,
            node_type,
            content: sn.content,
            fingerprint: sn.fingerprint,
            timestamp: sn.timestamp,
            emotions,
            prev_event: sn.prev_event,
            next_event: sn.next_event,
        };
        engine.nodes.insert(sn.id, node);

        // 如果是 Feature 节点，也更新关键词映射
        if node_type == NodeType::Feature {
            // 关键词映射会在下面的 keywords 加载中处理
        }
    }

    // 2. 加载关键词
    for kw in store.load_all_keywords()? {
        engine.keyword_to_node.insert(kw.keyword.clone(), kw.node_id);
        engine.feature_keywords.push(kw.keyword);
    }

    // 3. 加载记忆图边
    for e in store.load_all_edges(GraphType::Memory)? {
        let edges = engine.graph.entry(e.src).or_default();
        edges.push(GraphEdge {
            target_node_id: e.tgt,
            connection_strength: e.strength,
            edge_type: e.edge_type,
        });
    }

    // 4. 加载本体图边
    for e in store.load_all_edges(GraphType::Ontology)? {
        let edges = engine.ontology_graph.entry(e.src).or_default();
        edges.push(GraphEdge {
            target_node_id: e.tgt,
            connection_strength: e.strength,
            edge_type: e.edge_type,
        });
    }

    // 5. 加载向量
    for v in store.load_all_vectors()? {
        engine.chaos_store.add(v.id, v.fingerprint, v.vector);
    }

    Ok(())
}
