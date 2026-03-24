#![allow(dead_code)]

use std::path::Path;
use half::f16;
use crate::data::storage::ChaosFingerprint;
use crate::data::store::*;

// ============================================================================
// SQLite 存储后端
// ============================================================================

pub struct SqliteStore {
    conn: rusqlite::Connection,
}

impl SqliteStore {
    /// 打开或创建 SQLite 数据库
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, rusqlite::Error> {
        let conn = rusqlite::Connection::open(path)?;

        // WAL 模式 (高并发读 + 写性能)
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;

        // 创建表
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS nodes (
                id          INTEGER PRIMARY KEY,
                node_type   INTEGER NOT NULL,
                content     TEXT NOT NULL,
                fingerprint INTEGER NOT NULL,
                timestamp   INTEGER NOT NULL DEFAULT 0,
                emotions    BLOB,
                prev_event  INTEGER,
                next_event  INTEGER
            );

            CREATE TABLE IF NOT EXISTS edges (
                graph_type  INTEGER NOT NULL,
                src         INTEGER NOT NULL,
                tgt         INTEGER NOT NULL,
                strength    INTEGER NOT NULL,
                edge_type   INTEGER NOT NULL,
                PRIMARY KEY (graph_type, src, tgt)
            );

            CREATE TABLE IF NOT EXISTS keywords (
                keyword     TEXT PRIMARY KEY,
                node_id     INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS vectors (
                id          INTEGER PRIMARY KEY,
                fingerprint BLOB NOT NULL,
                vector      BLOB NOT NULL
            );
        ")?;

        Ok(Self { conn })
    }

    /// 创建内存数据库 (用于测试)
    #[allow(dead_code)]
    pub fn in_memory() -> Result<Self, rusqlite::Error> {
        let store = Self { conn: rusqlite::Connection::open_in_memory()? };
        store.conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
        store.conn.execute_batch("
            CREATE TABLE IF NOT EXISTS nodes (
                id INTEGER PRIMARY KEY, node_type INTEGER NOT NULL, content TEXT NOT NULL,
                fingerprint INTEGER NOT NULL, timestamp INTEGER NOT NULL DEFAULT 0,
                emotions BLOB, prev_event INTEGER, next_event INTEGER
            );
            CREATE TABLE IF NOT EXISTS edges (
                graph_type INTEGER NOT NULL, src INTEGER NOT NULL, tgt INTEGER NOT NULL,
                strength INTEGER NOT NULL, edge_type INTEGER NOT NULL,
                PRIMARY KEY (graph_type, src, tgt)
            );
            CREATE TABLE IF NOT EXISTS keywords (keyword TEXT PRIMARY KEY, node_id INTEGER NOT NULL);
            CREATE TABLE IF NOT EXISTS vectors (id INTEGER PRIMARY KEY, fingerprint BLOB NOT NULL, vector BLOB NOT NULL);
        ")?;
        Ok(store)
    }
}

impl PedsaStore for SqliteStore {
    type Error = rusqlite::Error;

    // ── 节点 ──

    fn save_node(&mut self, node: &StoredNode) -> Result<(), Self::Error> {
        self.conn.execute(
            "INSERT OR REPLACE INTO nodes (id, node_type, content, fingerprint, timestamp, emotions, prev_event, next_event) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                node.id,
                node.node_type,
                node.content,
                node.fingerprint as i64,
                node.timestamp as i64,
                &node.emotions,
                node.prev_event,
                node.next_event,
            ],
        )?;
        Ok(())
    }

    fn save_nodes_batch(&mut self, nodes: &[StoredNode]) -> Result<(), Self::Error> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT OR REPLACE INTO nodes (id, node_type, content, fingerprint, timestamp, emotions, prev_event, next_event) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
            )?;
            for n in nodes {
                stmt.execute(rusqlite::params![
                    n.id, n.node_type, n.content, n.fingerprint as i64,
                    n.timestamp as i64, &n.emotions, n.prev_event, n.next_event,
                ])?;
            }
        }
        tx.commit()
    }

    fn load_all_nodes(&self) -> Result<Vec<StoredNode>, Self::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, node_type, content, fingerprint, timestamp, emotions, prev_event, next_event FROM nodes"
        )?;
        let rows = stmt.query_map([], |row| {
            let emotions_blob: Vec<u8> = row.get::<_, Vec<u8>>(5).unwrap_or_default();
            Ok(StoredNode {
                id: row.get(0)?,
                node_type: row.get::<_, u8>(1)?,
                content: row.get(2)?,
                fingerprint: row.get::<_, i64>(3)? as u64,
                timestamp: row.get::<_, i64>(4)? as u64,
                emotions: emotions_blob,
                prev_event: row.get(6)?,
                next_event: row.get(7)?,
            })
        })?;
        rows.collect()
    }

    fn node_count(&self) -> Result<usize, Self::Error> {
        self.conn.query_row("SELECT COUNT(*) FROM nodes", [], |row| {
            row.get::<_, i64>(0).map(|c| c as usize)
        })
    }

    // ── 图边 ──

    fn save_edge(&mut self, graph_type: GraphType, edge: &StoredEdge) -> Result<(), Self::Error> {
        let gt = graph_type_to_int(graph_type);
        self.conn.execute(
            "INSERT OR REPLACE INTO edges (graph_type, src, tgt, strength, edge_type) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![gt, edge.src, edge.tgt, edge.strength as i32, edge.edge_type],
        )?;
        Ok(())
    }

    fn save_edges_batch(&mut self, graph_type: GraphType, edges: &[StoredEdge]) -> Result<(), Self::Error> {
        let gt = graph_type_to_int(graph_type);
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT OR REPLACE INTO edges (graph_type, src, tgt, strength, edge_type) VALUES (?1, ?2, ?3, ?4, ?5)"
            )?;
            for e in edges {
                stmt.execute(rusqlite::params![gt, e.src, e.tgt, e.strength as i32, e.edge_type])?;
            }
        }
        tx.commit()
    }

    fn load_all_edges(&self, graph_type: GraphType) -> Result<Vec<StoredEdge>, Self::Error> {
        let gt = graph_type_to_int(graph_type);
        let mut stmt = self.conn.prepare(
            "SELECT src, tgt, strength, edge_type FROM edges WHERE graph_type = ?1"
        )?;
        let rows = stmt.query_map([gt], |row| {
            Ok(StoredEdge {
                src: row.get(0)?,
                tgt: row.get(1)?,
                strength: row.get::<_, i32>(2)? as u16,
                edge_type: row.get::<_, u8>(3)?,
            })
        })?;
        rows.collect()
    }

    fn delete_edges(&mut self, graph_type: GraphType, src: i64, targets: &[i64]) -> Result<usize, Self::Error> {
        let gt = graph_type_to_int(graph_type);
        let mut deleted = 0;
        for &tgt in targets {
            deleted += self.conn.execute(
                "DELETE FROM edges WHERE graph_type = ?1 AND src = ?2 AND tgt = ?3",
                rusqlite::params![gt, src, tgt],
            )?;
        }
        Ok(deleted)
    }

    // ── 关键词 ──

    fn save_keyword(&mut self, kw: &StoredKeyword) -> Result<(), Self::Error> {
        self.conn.execute(
            "INSERT OR REPLACE INTO keywords (keyword, node_id) VALUES (?1, ?2)",
            rusqlite::params![kw.keyword, kw.node_id],
        )?;
        Ok(())
    }

    fn save_keywords_batch(&mut self, kws: &[StoredKeyword]) -> Result<(), Self::Error> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT OR REPLACE INTO keywords (keyword, node_id) VALUES (?1, ?2)"
            )?;
            for kw in kws {
                stmt.execute(rusqlite::params![kw.keyword, kw.node_id])?;
            }
        }
        tx.commit()
    }

    fn load_all_keywords(&self) -> Result<Vec<StoredKeyword>, Self::Error> {
        let mut stmt = self.conn.prepare("SELECT keyword, node_id FROM keywords")?;
        let rows = stmt.query_map([], |row| {
            Ok(StoredKeyword {
                keyword: row.get(0)?,
                node_id: row.get(1)?,
            })
        })?;
        rows.collect()
    }

    // ── 向量 ──

    fn save_vector(&mut self, v: &StoredVector) -> Result<(), Self::Error> {
        let fp_bytes: &[u8] = bytemuck::bytes_of(&v.fingerprint);
        let vec_bytes: &[u8] = bytemuck::cast_slice(&v.vector);
        self.conn.execute(
            "INSERT OR REPLACE INTO vectors (id, fingerprint, vector) VALUES (?1, ?2, ?3)",
            rusqlite::params![v.id, fp_bytes, vec_bytes],
        )?;
        Ok(())
    }

    fn save_vectors_batch(&mut self, vecs: &[StoredVector]) -> Result<(), Self::Error> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT OR REPLACE INTO vectors (id, fingerprint, vector) VALUES (?1, ?2, ?3)"
            )?;
            for v in vecs {
                let fp_bytes: &[u8] = bytemuck::bytes_of(&v.fingerprint);
                let vec_bytes: &[u8] = bytemuck::cast_slice(&v.vector);
                stmt.execute(rusqlite::params![v.id, fp_bytes, vec_bytes])?;
            }
        }
        tx.commit()
    }

    fn load_all_vectors(&self) -> Result<Vec<StoredVector>, Self::Error> {
        let mut stmt = self.conn.prepare("SELECT id, fingerprint, vector FROM vectors")?;
        let rows = stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let fp_blob: Vec<u8> = row.get(1)?;
            let vec_blob: Vec<u8> = row.get(2)?;

            let fingerprint = if fp_blob.len() == std::mem::size_of::<ChaosFingerprint>() {
                *bytemuck::from_bytes(&fp_blob)
            } else {
                ChaosFingerprint::default()
            };

            let vector: Vec<f16> = if vec_blob.len() % 2 == 0 {
                bytemuck::cast_slice(&vec_blob).to_vec()
            } else {
                Vec::new()
            };

            Ok(StoredVector { id, fingerprint, vector })
        })?;
        rows.collect()
    }

    // ── 生命周期 ──

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.conn.execute_batch("PRAGMA wal_checkpoint(PASSIVE);")?;
        Ok(())
    }

    fn clear(&mut self) -> Result<(), Self::Error> {
        self.conn.execute_batch("
            DELETE FROM nodes;
            DELETE FROM edges;
            DELETE FROM keywords;
            DELETE FROM vectors;
        ")?;
        Ok(())
    }
}

fn graph_type_to_int(gt: GraphType) -> i32 {
    match gt {
        GraphType::Memory => 0,
        GraphType::Ontology => 1,
    }
}
