use ahash::AHashMap;
use smallvec::SmallVec;
use half::f16;
use crate::data::storage::ChaosFingerprint;

// ============================================================================
// 核心数据结构
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Feature, // 特征锚点（关键词、实体）
    Event,   // 事件总结节点（记忆主体）
}

#[derive(Clone, Debug)]
pub struct GraphEdge {
    pub target_node_id: i64,
    pub connection_strength: u16,
    pub edge_type: u8, // V2: 0=关联, 1=因果, 2=顺序, 3=对比
}

pub struct Node {
    pub id: i64,
    pub node_type: NodeType,
    pub content: String,       // 对于 Event 是总结，对于 Feature 是关键词
    pub fingerprint: u64,      // 语义指纹
    
    // V2 新增字段
    pub timestamp: u64,        // Unix 时间戳
    pub emotions: SmallVec<[u8; 8]>, // 情感矢量 (8维)
    pub prev_event: Option<i64>,     // 时序前驱
    pub next_event: Option<i64>,     // 时序后继
}

// ============================================================================
// ChaosStore: SoA 混沌向量存储
// ============================================================================

pub struct ChaosStore {
    pub ids: Vec<i64>,
    pub fingerprints: Vec<ChaosFingerprint>,
    pub vectors: Vec<Vec<f16>>,
    pub id_to_index: AHashMap<i64, usize>,
}

impl ChaosStore {
    pub fn new() -> Self {
        Self {
            ids: Vec::new(),
            fingerprints: Vec::new(),
            vectors: Vec::new(),
            id_to_index: AHashMap::new(),
        }
    }

    pub fn add(&mut self, id: i64, fp: ChaosFingerprint, vec: Vec<f16>) {
        if !self.id_to_index.contains_key(&id) {
            let idx = self.ids.len();
            self.ids.push(id);
            self.fingerprints.push(fp);
            self.vectors.push(vec);
            self.id_to_index.insert(id, idx);
        }
    }
}

// ============================================================================
// 异步任务接口
// ============================================================================

#[allow(dead_code)]
pub trait AsyncTaskInterface {
    fn schedule_maintenance(&self, context: &str);
}

pub struct MockAsyncTask;
impl AsyncTaskInterface for MockAsyncTask {
    fn schedule_maintenance(&self, _context: &str) {
        // Placeholder
    }
}
