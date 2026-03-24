//! PyO3 Python 绑定模块
//!
//! 通过 `maturin develop` 构建后，Python 侧使用:
//! ```python
//! import pedsa
//! engine = pedsa.Engine()
//! engine.add_event(1, "Rust 在嵌入式领域取得突破")
//! engine.compile()
//! results = engine.retrieve("Rust 内存安全")
//! ```

use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;

use crate::core::engine::AdvancedEngine;
use crate::data::store::{self, GraphType, StoredNode, StoredEdge, StoredKeyword};
use crate::data::sqlite_store::SqliteStore;
use crate::data::store::PedsaStore;

// ============================================================================
// PedsaEngine: AdvancedEngine 的 Python 包装
// ============================================================================

/// PEDSA 记忆检索引擎 (Python 绑定)
#[pyclass(name = "Engine")]
pub struct PedsaEngine {
    inner: AdvancedEngine,
}

#[pymethods]
impl PedsaEngine {
    /// 创建新的引擎实例
    #[new]
    fn new() -> Self {
        Self { inner: AdvancedEngine::new() }
    }

    /// 加载嵌入模型
    ///
    /// 模型路径为 GGUF 格式的 BGE-M3 模型目录
    /// 如果加载失败返回 False，成功返回 True
    fn load_embedding_model(&mut self) -> PyResult<bool> {
        match crate::ml::embedding::CandleModel::new() {
            Ok(model) => {
                let dim = model.dimension;
                self.inner.embedding_model = Some(model);
                println!("已加载 {}维 Candle 向量模型 (BGE-M3 GGUF)", dim);
                Ok(true)
            }
            Err(e) => {
                println!("模型加载失败: {}", e);
                Ok(false)
            }
        }
    }

    /// 加载 GLiNER NER 模型 (需要 gliner feature)
    fn load_gliner_model(&mut self, model_dir: &str) -> PyResult<bool> {
        #[cfg(feature = "gliner")]
        {
            match crate::ml::gliner_ner::GlinerEngine::new(model_dir) {
                Ok(engine) => {
                    self.inner.gliner_engine = Some(engine);
                    Ok(true)
                }
                Err(e) => {
                    println!("GLiNER 加载失败: {}", e);
                    Ok(false)
                }
            }
        }
        #[cfg(not(feature = "gliner"))]
        {
            let _ = model_dir;
            println!("GLiNER feature 未启用，跳过加载");
            Ok(false)
        }
    }

    /// 添加特征节点 (关键词锚点)
    fn add_feature(&mut self, id: i64, keyword: &str) {
        self.inner.add_feature(id, keyword);
    }

    /// 添加事件节点 (记忆主体)
    ///
    /// 参数:
    ///   id: 节点 ID
    ///   summary: 事件摘要文本
    #[pyo3(signature = (id, summary))]
    fn add_event(&mut self, id: i64, summary: &str) {
        self.inner.add_event(id, summary, None, None);
    }

    /// 建立节点关联
    fn add_edge(&mut self, src: i64, tgt: i64, weight: f32) {
        self.inner.add_edge(src, tgt, weight);
    }

    /// 维护本体关联 (Ontology)
    fn maintain_ontology(&mut self, source: &str, target: &str, relation_type: &str, strength: f32) {
        self.inner.maintain_ontology(source, target, relation_type, strength);
    }

    /// 编译引擎 (构建 AC 自动机、计算入度、建立时序脊梁)
    ///
    /// 在添加完所有数据后必须调用此方法
    fn compile(&mut self) {
        self.inner.compile();
    }

    /// 执行检索
    ///
    /// 参数:
    ///   query: 查询文本
    ///   ref_time: 参考时间戳 (用于相对时间解析，0=忽略)
    ///   chaos_level: 混沌程度 (0.0=纯理性, 1.0=纯混沌)
    ///
    /// 返回:
    ///   [(id, score), ...] 按分数降序排列
    #[pyo3(signature = (query, ref_time=0, chaos_level=0.0))]
    fn retrieve(&self, query: &str, ref_time: u64, chaos_level: f32) -> Vec<(i64, f32)> {
        self.inner.retrieve(query, ref_time, chaos_level)
    }

    /// 加载内置标准数据集
    fn load_standard_data(&mut self) {
        self.inner.load_standard_data();
    }

    /// 获取节点内容
    fn get_node_content(&self, id: i64) -> Option<String> {
        self.inner.nodes.get(&id).map(|n| n.content.clone())
    }

    /// 获取节点总数
    fn node_count(&self) -> usize {
        self.inner.nodes.len()
    }

    /// 获取特征关键词总数
    fn feature_count(&self) -> usize {
        self.inner.feature_keywords.len()
    }

    /// 全局衰减与剪枝
    fn apply_decay(&mut self, decay_rate: f32, threshold: u16) -> usize {
        self.inner.apply_global_decay_and_pruning(decay_rate, threshold)
    }

    /// 触发逻辑仲裁 (返回 1-hop 子图上下文)
    fn trigger_arbitration(&self, source: &str) -> Option<String> {
        self.inner.trigger_arbitration(source)
    }

    /// 应用仲裁结果 (删除冲突关联)
    fn apply_arbitration(&mut self, source: &str, delete_targets: Vec<String>) {
        self.inner.apply_arbitration(source, delete_targets);
    }

    /// 导出引擎数据到 SQLite
    fn export_to_sqlite(&self, path: &str) -> PyResult<()> {
        let mut store = SqliteStore::open(path)
            .map_err(|e| PyRuntimeError::new_err(format!("SQLite open error: {}", e)))?;
        store::export_engine_to_store(&self.inner, &mut store)
            .map_err(|e| PyRuntimeError::new_err(format!("Export error: {}", e)))?;
        Ok(())
    }

    /// 从 SQLite 导入数据到引擎
    fn import_from_sqlite(&mut self, path: &str) -> PyResult<()> {
        let store = SqliteStore::open(path)
            .map_err(|e| PyRuntimeError::new_err(format!("SQLite open error: {}", e)))?;
        store::import_store_to_engine(&store, &mut self.inner)
            .map_err(|e| PyRuntimeError::new_err(format!("Import error: {}", e)))?;
        Ok(())
    }

    // ── 图谱构建辅助 ──

    /// 获取或创建特征节点 (自动分配 ID)
    ///
    /// 如果关键词已存在则返回已有 ID，否则创建新节点并返回 ID。
    /// 停用词返回 -1。
    fn get_or_create_feature(&mut self, word: &str) -> i64 {
        self.inner.get_or_create_feature(word)
    }

    /// 统一维护接口 (upsert/replace)
    ///
    /// action: "upsert" 或 "replace"
    #[pyo3(signature = (action, source, target, relation_type, strength, reason=""))]
    fn execute_maintenance(&mut self, action: &str, source: &str, target: &str, relation_type: &str, strength: f32, reason: &str) -> Option<String> {
        self.inner.execute_maintenance(action, source, target, relation_type, strength, reason)
    }

    // ── 图谱查询 ──

    /// 获取节点完整信息
    ///
    /// 返回 dict: {id, node_type, content, fingerprint, timestamp, emotions, prev_event, next_event}
    /// 如果节点不存在返回 None
    fn get_node(&self, id: i64) -> Option<PyObject> {
        Python::with_gil(|py| {
            let node = self.inner.nodes.get(&id)?;
            let dict = pyo3::types::PyDict::new(py);
            dict.set_item("id", node.id).ok()?;
            dict.set_item("node_type", match node.node_type {
                crate::core::types::NodeType::Feature => "feature",
                crate::core::types::NodeType::Event => "event",
            }).ok()?;
            dict.set_item("content", &node.content).ok()?;
            dict.set_item("fingerprint", node.fingerprint).ok()?;
            dict.set_item("timestamp", node.timestamp).ok()?;
            dict.set_item("emotions", node.emotions.to_vec()).ok()?;
            dict.set_item("prev_event", node.prev_event).ok()?;
            dict.set_item("next_event", node.next_event).ok()?;
            Some(dict.into())
        })
    }

    /// 获取记忆图中指定节点的邻居列表
    ///
    /// 返回 [(target_id, strength, edge_type), ...]
    fn get_edges(&self, node_id: i64) -> Vec<(i64, u16, u8)> {
        self.inner.graph.get(&node_id)
            .map(|edges| edges.iter().map(|e| (e.target_node_id, e.connection_strength, e.edge_type)).collect())
            .unwrap_or_default()
    }

    /// 获取本体图中指定节点的邻居列表
    ///
    /// 返回 [(target_id, strength, edge_type), ...]
    fn get_ontology_edges(&self, node_id: i64) -> Vec<(i64, u16, u8)> {
        self.inner.ontology_graph.get(&node_id)
            .map(|edges| edges.iter().map(|e| (e.target_node_id, e.connection_strength, e.edge_type)).collect())
            .unwrap_or_default()
    }

    /// 获取所有节点 ID
    fn all_node_ids(&self) -> Vec<i64> {
        self.inner.nodes.keys().copied().collect()
    }

    /// 获取所有特征关键词
    fn all_feature_keywords(&self) -> Vec<String> {
        self.inner.feature_keywords.clone()
    }

    /// 根据关键词查找节点 ID
    fn keyword_to_id(&self, keyword: &str) -> Option<i64> {
        self.inner.keyword_to_node.get(&keyword.to_lowercase()).copied()
    }

    fn __repr__(&self) -> String {
        format!(
            "PedsaEngine(nodes={}, features={}, has_embedding={})",
            self.inner.nodes.len(),
            self.inner.feature_keywords.len(),
            self.inner.embedding_model.is_some(),
        )
    }
}

// ============================================================================
// PedsaSqliteStore: SqliteStore 的 Python 包装
// ============================================================================

/// SQLite 持久化存储后端 (Python 绑定)
#[pyclass(unsendable, name = "SqliteStore")]
pub struct PedsaSqliteStore {
    inner: SqliteStore,
}

#[pymethods]
impl PedsaSqliteStore {
    /// 打开或创建 SQLite 数据库
    #[new]
    fn new(path: &str) -> PyResult<Self> {
        let store = SqliteStore::open(path)
            .map_err(|e| PyRuntimeError::new_err(format!("SQLite open error: {}", e)))?;
        Ok(Self { inner: store })
    }

    /// 保存单个节点
    fn save_node(&mut self, id: i64, node_type: u8, content: &str, fingerprint: u64, timestamp: u64) -> PyResult<()> {
        let node = StoredNode {
            id, node_type, content: content.to_string(), fingerprint, timestamp,
            emotions: Vec::new(), prev_event: None, next_event: None,
        };
        self.inner.save_node(&node)
            .map_err(|e| PyRuntimeError::new_err(format!("{}", e)))
    }

    /// 保存图边
    fn save_edge(&mut self, graph_type: u8, src: i64, tgt: i64, strength: u16, edge_type: u8) -> PyResult<()> {
        let gt = if graph_type == 0 { GraphType::Memory } else { GraphType::Ontology };
        let edge = StoredEdge { src, tgt, strength, edge_type };
        self.inner.save_edge(gt, &edge)
            .map_err(|e| PyRuntimeError::new_err(format!("{}", e)))
    }

    /// 保存关键词
    fn save_keyword(&mut self, keyword: &str, node_id: i64) -> PyResult<()> {
        let kw = StoredKeyword { keyword: keyword.to_string(), node_id };
        self.inner.save_keyword(&kw)
            .map_err(|e| PyRuntimeError::new_err(format!("{}", e)))
    }

    /// 获取节点总数
    fn node_count(&self) -> PyResult<usize> {
        self.inner.node_count()
            .map_err(|e| PyRuntimeError::new_err(format!("{}", e)))
    }

    /// 刷新到磁盘
    fn flush(&mut self) -> PyResult<()> {
        self.inner.flush()
            .map_err(|e| PyRuntimeError::new_err(format!("{}", e)))
    }

    /// 清空所有数据
    fn clear(&mut self) -> PyResult<()> {
        self.inner.clear()
            .map_err(|e| PyRuntimeError::new_err(format!("{}", e)))
    }

    fn __repr__(&self) -> String {
        let count = self.inner.node_count().unwrap_or(0);
        format!("PedsaSqliteStore(nodes={})", count)
    }
}

// ============================================================================
// Python 模块定义
// ============================================================================

/// PEDSA - Brain-inspired RAG-less Memory Engine
#[pymodule]
fn pedsa(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PedsaEngine>()?;
    m.add_class::<PedsaSqliteStore>()?;
    Ok(())
}
