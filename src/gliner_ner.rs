//! GLiNER-X-Base 集成模块
//! 
//! 职责: 实体类型识别 + 时间 span 定位
//! 架构: jieba-rs 预分词 → gline-rs ONNX 推理 → 结果映射到 SimHash 分区
//!
//! 设计原则:
//! - 可选依赖: GlinerEngine 为 Option<>，不影响现有功能
//! - 编译阶段预热: 在 compile() 中初始化，避免首次检索延迟
//! - jieba 自定义词: 从 Ontology Feature 节点动态加载

use gliner::model::GLiNER;
use gliner::model::params::Parameters;
use gliner::model::pipeline::span::SpanMode;
use gliner::model::input::text::TextInput;
use orp::params::RuntimeParameters;
use jieba_rs::Jieba;
use std::path::Path;

// ============================================================================
// 常量
// ============================================================================

/// PEDSA 实体类型标签 (对应 SimHash [56-63] TYPE 区)
const TYPE_LABELS: &[&str] = &[
    "person",           // TYPE_PERSON  0x01
    "technology",       // TYPE_TECH    0x02
    "event",            // TYPE_EVENT   0x03
    "location",         // TYPE_LOCATION 0x04
    "object",           // TYPE_OBJECT  0x05
    "value or belief",  // TYPE_VALUES  0x06
];

/// 时间实体标签 (对应 SimHash [32-47] TEMPORAL 区)
const TIME_LABELS: &[&str] = &[
    "date",
    "time",
    "relative time",
];

/// 默认置信度阈值
const DEFAULT_THRESHOLD: f32 = 0.3;

// ============================================================================
// 数据结构
// ============================================================================

/// GLiNER 提取的单个实体
#[derive(Debug, Clone)]
pub struct ExtractedEntity {
    /// 提取的文本 span (已去除空格)
    pub text: String,
    /// GLiNER 标签 (如 "person", "date")
    pub label: String,
    /// 置信度分数
    pub score: f32,
}

/// GLiNER 引擎封装
pub struct GlinerEngine {
    model: GLiNER<SpanMode>,
    pub jieba: Jieba,
}

impl GlinerEngine {
    /// 从 ONNX 模型目录初始化
    /// 
    /// 目录需包含:
    /// - model.onnx      (ONNX 模型文件)
    /// - tokenizer.json   (HuggingFace tokenizer)
    pub fn new(model_dir: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let tokenizer_path = format!("{}/tokenizer.json", model_dir);
        let model_path = format!("{}/model.onnx", model_dir);
        
        if !Path::new(&tokenizer_path).exists() {
            return Err(format!("tokenizer not found: {}", tokenizer_path).into());
        }
        if !Path::new(&model_path).exists() {
            return Err(format!("model not found: {}", model_path).into());
        }
        
        let params = Parameters::default();
        let runtime = RuntimeParameters::default();
        
        let model = GLiNER::<SpanMode>::new(
            params, runtime,
            &tokenizer_path, &model_path,
        ).map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { 
            format!("GLiNER model load error: {}", e).into() 
        })?;
        
        let jieba = Jieba::new();
        
        Ok(Self { model, jieba })
    }
    
    /// 添加自定义词到 jieba 词典 (从 Ontology Feature 节点加载)
    pub fn add_custom_word(&mut self, word: &str) {
        self.jieba.add_word(word, Some(100_000), None);
    }
    
    /// jieba 预分词: 中文文本 → 空格分隔的 tokens
    fn preprocess(&self, text: &str) -> String {
        self.jieba.cut(text, false).join(" ")
    }
    
    /// 提取实体类型 (SimHash [56-63] TYPE 区)
    pub fn extract_types(&self, text: &str) -> Vec<ExtractedEntity> {
        self.extract_with_labels(text, TYPE_LABELS)
    }
    
    /// 提取时间 span (SimHash [32-47] TEMPORAL 区)
    pub fn extract_times(&self, text: &str) -> Vec<ExtractedEntity> {
        self.extract_with_labels(text, TIME_LABELS)
    }
    
    /// 一站式提取 (类型 + 时间)
    pub fn extract_all(&self, text: &str) -> (Vec<ExtractedEntity>, Vec<ExtractedEntity>) {
        let segmented = self.preprocess(text);
        
        // 合并标签，一次推理提取全部
        let mut all_labels: Vec<&str> = Vec::new();
        all_labels.extend_from_slice(TYPE_LABELS);
        all_labels.extend_from_slice(TIME_LABELS);
        
        let entities = self.run_inference(&segmented, &all_labels);
        
        // 按类别拆分
        let mut types = Vec::new();
        let mut times = Vec::new();
        
        for e in entities {
            if TYPE_LABELS.contains(&e.label.as_str()) {
                types.push(e);
            } else if TIME_LABELS.contains(&e.label.as_str()) {
                times.push(e);
            }
        }
        
        (types, times)
    }
    
    /// 内部: 带指定标签的提取
    fn extract_with_labels(&self, text: &str, labels: &[&str]) -> Vec<ExtractedEntity> {
        let segmented = self.preprocess(text);
        self.run_inference(&segmented, labels)
    }
    
    /// 内部: 执行 ONNX 推理
    fn run_inference(&self, segmented_text: &str, labels: &[&str]) -> Vec<ExtractedEntity> {
        let input = match TextInput::from_str(&[segmented_text], labels) {
            Ok(inp) => inp,
            Err(_) => return Vec::new(),
        };
        
        let output = match self.model.inference(input) {
            Ok(out) => out,
            Err(_) => return Vec::new(),
        };
        
        let mut results = Vec::new();
        // output.spans: Vec<Vec<Span>> - one per input document
        for doc_spans in &output.spans {
            for span in doc_spans {
                if span.probability() >= DEFAULT_THRESHOLD {
                    results.push(ExtractedEntity {
                        // 去除 jieba 插入的空格，恢复原文
                        text: span.text().replace(' ', ""),
                        label: span.class().to_string(),
                        score: span.probability(),
                    });
                }
            }
        }
        
        results
    }
}

// ============================================================================
// SimHash 映射函数
// ============================================================================

/// GLiNER 标签 → PEDSA TYPE 常量
pub fn label_to_type_val(label: &str) -> u8 {
    match label {
        "person"          => 0x01, // TYPE_PERSON
        "technology"      => 0x02, // TYPE_TECH
        "event"           => 0x03, // TYPE_EVENT
        "location"        => 0x04, // TYPE_LOCATION
        "object"          => 0x05, // TYPE_OBJECT
        "value or belief" => 0x06, // TYPE_VALUES
        _                 => 0x00, // TYPE_UNKNOWN
    }
}

/// 从类型实体列表中选取最佳类型
/// 策略: 取置信度最高的实体的类型
pub fn best_type_val(entities: &[ExtractedEntity]) -> u8 {
    entities.iter()
        .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal))
        .map(|e| label_to_type_val(&e.label))
        .unwrap_or(0x00)
}

/// 从时间 span 解析为 Unix 时间戳
/// ref_time: PEDSA 的参考时间 (现实/叙事时间)
pub fn span_to_timestamp(span: &str, ref_time: u64) -> u64 {
    let s = span.to_lowercase();
    let s = s.trim();
    
    // --- 精确相对时间 ---
    if s == "今天" || s == "今日" || s == "today" { return ref_time; }
    if s == "昨天" || s == "昨日" || s == "yesterday" { return ref_time.saturating_sub(86400); }
    if s == "前天" || s == "前日" { return ref_time.saturating_sub(172800); }
    if s == "大前天" { return ref_time.saturating_sub(259200); }
    if s == "刚才" || s == "刚刚" || s == "just now" { return ref_time.saturating_sub(60); }
    if s == "最近" || s == "recently" { return ref_time.saturating_sub(259200); }
    
    // --- 组合: "昨天下午", "今天早上" ---
    if s.starts_with("昨天") { return ref_time.saturating_sub(86400); }
    if s.starts_with("前天") { return ref_time.saturating_sub(172800); }
    if s.starts_with("今天") { return ref_time; }
    
    // --- 上周/上月/去年 ---
    if s.starts_with("上周") || s == "上星期" { return ref_time.saturating_sub(604800); }
    if s == "上个月" || s == "上月" { return ref_time.saturating_sub(2592000); }
    if s == "去年" { return ref_time.saturating_sub(31536000); }
    if s == "前年" { return ref_time.saturating_sub(63072000); }
    
    // --- 时间段 ---
    if s == "早上" || s == "上午" { return ref_time; }
    if s == "下午" { return ref_time; }
    if s == "晚上" { return ref_time; }
    
    // --- N天前/N周前 (中文数字) ---
    let cn_num = |c: char| -> Option<u64> {
        match c {
            '一' | '1' => Some(1), '二' | '两' | '2' => Some(2),
            '三' | '3' => Some(3), '四' | '4' => Some(4),
            '五' | '5' => Some(5), '六' | '6' => Some(6),
            '七' | '7' => Some(7), '八' | '8' => Some(8),
            '九' | '9' => Some(9), '十' => Some(10),
            _ => None,
        }
    };
    
    // "三天前", "3天前"
    if s.ends_with("天前") {
        if let Some(n) = s.chars().next().and_then(cn_num) {
            return ref_time.saturating_sub(n * 86400);
        }
    }
    // "两周前", "2周前"
    if s.ends_with("周前") {
        if let Some(n) = s.chars().next().and_then(cn_num) {
            return ref_time.saturating_sub(n * 604800);
        }
    }
    // "三个月前"
    if s.ends_with("月前") || s.ends_with("个月前") {
        if let Some(n) = s.chars().next().and_then(cn_num) {
            return ref_time.saturating_sub(n * 2592000);
        }
    }
    
    // --- 绝对日期 "2025年3月15日" ---
    if let Some(ts) = parse_absolute_date(s) {
        return ts;
    }
    
    // --- 具体时刻 "8点" ---
    if s.ends_with("点") {
        // 仅标记为当天，不做精确小时解析 (SimHash 时间区精度有限)
        return ref_time;
    }
    
    0 // 无法解析
}

/// 解析绝对日期字符串
fn parse_absolute_date(s: &str) -> Option<u64> {
    // 匹配 "2025年3月15日" 或 "2025年3月"
    let mut year = 0u64;
    let mut month = 1u64;
    let mut day = 1u64;
    
    if let Some(y_end) = s.find('年') {
        if y_end >= 4 {
            if let Ok(y) = s[y_end-4..y_end].parse::<u64>() {
                year = y;
            }
        }
        let rest = &s[y_end + '年'.len_utf8()..];
        if let Some(m_end) = rest.find('月') {
            if let Ok(m) = rest[..m_end].trim().parse::<u64>() {
                month = m;
            }
            let rest2 = &rest[m_end + '月'.len_utf8()..];
            if let Some(d_end) = rest2.find('日') {
                if let Ok(d) = rest2[..d_end].trim().parse::<u64>() {
                    day = d;
                }
            }
        }
    }
    
    if year > 1970 {
        Some((year - 1970) * 31536000 + month * 2592000 + day * 86400)
    } else {
        None
    }
}

/// 从时间实体列表中选取最佳时间戳
/// 策略: 取能被解析且置信度最高的 span
pub fn best_timestamp(entities: &[ExtractedEntity], ref_time: u64) -> u64 {
    entities.iter()
        .filter_map(|e| {
            let ts = span_to_timestamp(&e.text, ref_time);
            if ts > 0 { Some((ts, e.score)) } else { None }
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(ts, _)| ts)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    const REF_TIME: u64 = 1711267200; // 2024-03-24
    
    #[test]
    fn test_span_to_timestamp_relative() {
        assert_eq!(span_to_timestamp("昨天", REF_TIME), REF_TIME - 86400);
        assert_eq!(span_to_timestamp("前天", REF_TIME), REF_TIME - 172800);
        assert_eq!(span_to_timestamp("上周", REF_TIME), REF_TIME - 604800);
        assert_eq!(span_to_timestamp("去年", REF_TIME), REF_TIME - 31536000);
        assert_eq!(span_to_timestamp("昨天下午", REF_TIME), REF_TIME - 86400);
    }
    
    #[test]
    fn test_span_to_timestamp_cn_num() {
        assert_eq!(span_to_timestamp("三天前", REF_TIME), REF_TIME - 3 * 86400);
        assert_eq!(span_to_timestamp("两周前", REF_TIME), REF_TIME - 2 * 604800);
    }
    
    #[test]
    fn test_label_to_type() {
        assert_eq!(label_to_type_val("person"), 0x01);
        assert_eq!(label_to_type_val("technology"), 0x02);
        assert_eq!(label_to_type_val("unknown_label"), 0x00);
    }
}
