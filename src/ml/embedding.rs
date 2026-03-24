use candle_core::{Device, Tensor};
use crate::ml::inference_engine::QuantizedBertModel as QBertModel;
use tokenizers::{Tokenizer, PaddingParams, TruncationParams};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

/// 基于 Candle 的嵌入模型 (支持 BGE-Small-ZH GGUF)
/// 
/// 支持的模型:
/// 1. BGE-Small-ZH (GGUF 量化, 512 维)
pub struct CandleModel {
    model: Arc<Mutex<QBertModel>>,
    tokenizer: Tokenizer,
    pub dimension: usize,
}

impl CandleModel {
    /// 初始化模型 (仅支持 BGE-Small-ZH GGUF)
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // 优先级 1: BGE-Small-ZH GGUF F16 (最佳性能, ~0.2s 延迟)
        // 注意: 对于小模型，F16 比 Q8_0 更快，因为反量化开销更小
        let bge_small_f16 = "models/bge-small-zh-v1.5-gguf/bge-small-zh-v1.5-f16.gguf";
        if PathBuf::from(bge_small_f16).exists() {
             println!("🔍 Found local model: {}", bge_small_f16);
             return Self::load_quantized_gguf(bge_small_f16);
        }

        // 优先级 2: BGE-Small-ZH GGUF Q8_0 (体积更小, 推理较慢 ~1.8s)
        let bge_small_q8 = "models/bge-small-zh-v1.5-gguf/bge-small-zh-v1.5-q8_0.gguf";
        if PathBuf::from(bge_small_q8).exists() {
             println!("🔍 Found local model: {}", bge_small_q8);
             return Self::load_quantized_gguf(bge_small_q8);
        }

        Err("❌ No supported model found. Please download BGE-Small GGUF.".into())
    }

    /// 加载量化 GGUF 模型
    pub fn load_quantized_gguf(model_dir: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let model_path = PathBuf::from(model_dir);

        // 检查 model_dir 是文件还是目录
        let is_file = model_path.extension().map_or(false, |ext| ext == "gguf");

        // GGUF 模型文件
        let weights_filename = if is_file {
            model_path.clone()
        } else {
            model_path.join("bge-m3-q4_k_m.gguf")
        };

        let search_dir = if is_file {
            model_path.parent().unwrap()
        } else {
            &model_path
        };

        // Tokenizer 配置
        let tokenizer_filename = if search_dir.join("tokenizer.json").exists() {
            search_dir.join("tokenizer.json")
        } else if search_dir.parent().unwrap().join("tokenizer.json").exists() {
             search_dir.parent().unwrap().join("tokenizer.json")
        } else {
             // 尝试向上查找一级
             let parent = search_dir.parent().unwrap();
             if parent.parent().unwrap().join("tokenizer.json").exists() {
                 parent.parent().unwrap().join("tokenizer.json")
             } else {
                 return Err("❌ tokenizer.json not found in model directory".into());
             }
        };

        println!("🏗️ Initializing Candle Quantized Model...");
        println!("📂 Loading weights from: {:?}", weights_filename);

        if !weights_filename.exists() {
            return Err(format!("❌ Weights file not found: {:?}", weights_filename).into());
        }

        let model = QBertModel::new(weights_filename.to_str().unwrap())?;
        let tokenizer = Tokenizer::from_file(tokenizer_filename).map_err(|e| e.to_string())?;

        // 从模型获取维度
        let hidden_size = model.hidden_size(); 
        
        println!("✅ Model loaded successfully. Hidden size: {}", hidden_size);

        Ok(Self {
            model: Arc::new(Mutex::new(model)),
            tokenizer,
            dimension: hidden_size,
        })
    }

    /// 执行向量化 (推理)
    pub fn vectorize_weighted(&self, text: &str, _weighted_ranges: &[(usize, usize, f32)]) -> Option<Vec<f32>> {
        let device = Device::Cpu;
        let mut tokenizer = self.tokenizer.clone();
        
        // 配置填充
        if let Some(pp) = tokenizer.get_padding_mut() {
            pp.strategy = tokenizers::PaddingStrategy::BatchLongest;
        } else {
            let pp = PaddingParams {
                strategy: tokenizers::PaddingStrategy::BatchLongest,
                ..Default::default()
            };
            tokenizer.with_padding(Some(pp));
        }

        // 配置截断
        if let Some(tp) = tokenizer.get_truncation_mut() {
            tp.max_length = 512;
        } else {
            let tp = TruncationParams {
                max_length: 512,
                ..Default::default()
            };
            let _ = tokenizer.with_truncation(Some(tp));
        }

        // 分词
        let tokens = match tokenizer.encode(text, true) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("❌ Tokenizer error: {}", e);
                return None;
            }
        };
        let token_ids = match Tensor::new(tokens.get_ids(), &device) {
            Ok(t) => match t.unsqueeze(0) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("❌ Tensor unsqueeze error: {}", e);
                    return None;
                }
            },
            Err(e) => {
                eprintln!("❌ Tensor creation error: {}", e);
                return None;
            }
        };
        let token_type_ids = match token_ids.zeros_like() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("❌ Tensor zeros_like error: {}", e);
                return None;
            }
        };

        // 前向传播
        let embeddings = {
            let model = self.model.lock().unwrap();
            match model.forward(&token_ids, Some(&token_type_ids)) {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("❌ Quantized Model forward error: {}", e);
                    return None;
                }
            }
        };

        // 池化策略: CLS Token (索引 0)
        // embeddings 形状: [1, seq_len, hidden_size]
        let cls_embedding = match embeddings.get(0) {
            Ok(e) => match e.get(0) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("❌ Embedding get(0) error: {}", e);
                    return None;
                }
            },
            Err(e) => {
                eprintln!("❌ Embeddings get(0) error: {}", e);
                return None;
            }
        };
        
        // 归一化 (L2)
        let l2_norm = match cls_embedding.sqr() {
            Ok(s) => match s.sum_all() {
                Ok(sum) => match sum.sqrt() {
                    Ok(sqrt) => sqrt,
                    Err(e) => {
                        eprintln!("❌ Sqrt error: {}", e);
                        return None;
                    }
                },
                Err(e) => {
                    eprintln!("❌ Sum all error: {}", e);
                    return None;
                }
            },
            Err(e) => {
                eprintln!("❌ Sqr error: {}", e);
                return None;
            }
        };
        let normalized = match cls_embedding.broadcast_div(&l2_norm) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("❌ Normalization error: {}", e);
                return None;
            }
        };

        let vec: Vec<f32> = match normalized.flatten_all() {
            Ok(f) => match f.to_vec1() {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("❌ To vec1 error: {}", e);
                    return None;
                }
            },
            Err(e) => {
                eprintln!("❌ Flatten all error: {}", e);
                return None;
            }
        };
        
        Some(vec)
    }

    /// 兼容接口
    pub fn vectorize(&self, text: &str) -> Option<Vec<f32>> {
        self.vectorize_weighted(text, &[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_model_loading() {
        println!("Testing local model loading...");
        // 如果测试环境中没有完整路径，则使用简化路径进行测试
        // 但我们期望它能工作。
        let model = CandleModel::new();
        if let Ok(m) = model {
            println!("Model loaded successfully!");
            let vec = m.vectorize("Hello world");
            assert!(vec.is_some());
            let v = vec.unwrap();
            assert_eq!(v.len(), 512);
            println!("Vector sample: {:?}", &v[0..5]);
        } else {
            eprintln!("Model failed to load. Ensure model files are in 'models/bge-m3-gguf/Embedding-GGUF/bge-m3-Q4_K_M-GGUF'");
        }
    }
}
