use candle_core::quantized::{gguf_file, QMatMul};
use candle_core::{Device, Tensor, DType, Result, Module};
use candle_nn::LayerNorm;

#[derive(Debug)]
pub struct QuantizedBertLayer {
    attention_self_query: QMatMul,
    attention_self_query_bias: Option<Tensor>,
    attention_self_key: QMatMul,
    attention_self_key_bias: Option<Tensor>,
    attention_self_value: QMatMul,
    attention_self_value_bias: Option<Tensor>,
    attention_output_dense: QMatMul,
    attention_output_dense_bias: Option<Tensor>,
    attention_output_norm: LayerNorm,
    intermediate_dense: QMatMul,
    intermediate_dense_bias: Option<Tensor>,
    output_dense: QMatMul,
    output_dense_bias: Option<Tensor>,
    output_norm: LayerNorm,
    num_heads: usize,
}

#[derive(Debug)]
pub struct QuantizedBertModel {
    embeddings: Tensor, // [vocab_size, hidden_size]
    position_embeddings: Tensor, // [max_position_embeddings, hidden_size]
    token_type_embeddings: Option<Tensor>, // [type_vocab_size, hidden_size]
    embeddings_norm: LayerNorm,
    layers: Vec<QuantizedBertLayer>,
    pub device: Device,
}

impl QuantizedBertLayer {
    fn forward(&self, x: &Tensor, mask: &Tensor) -> Result<Tensor> {
        // Attention
        // x: [batch, seq_len, hidden]
        let (batch, seq_len, hidden) = x.dims3()?;
        
        let q = self.attention_self_query.forward(x)?;
        let q = match &self.attention_self_query_bias {
            Some(b) => q.broadcast_add(b)?,
            None => q,
        };
        
        let k = self.attention_self_key.forward(x)?;
        let k = match &self.attention_self_key_bias {
            Some(b) => k.broadcast_add(b)?,
            None => k,
        };
        
        let v = self.attention_self_value.forward(x)?;
        let v = match &self.attention_self_value_bias {
            Some(b) => v.broadcast_add(b)?,
            None => v,
        };

        // 重塑以进行多头注意力 (Multi-head Attention)
        // num_heads = 16, head_dim = 64 (1024/16)
        let num_heads = self.num_heads;
        let head_dim = hidden / num_heads;
        
        let q = q.reshape((batch, seq_len, num_heads, head_dim))?.transpose(1, 2)?; // [batch, heads, seq, head_dim]
        let k = k.reshape((batch, seq_len, num_heads, head_dim))?.transpose(1, 2)?;
        let v = v.reshape((batch, seq_len, num_heads, head_dim))?.transpose(1, 2)?;

        // 缩放点积注意力 (Scaled Dot-Product Attention)
        // score = q @ k.t() / sqrt(head_dim)
        let scale = 1.0 / (head_dim as f64).sqrt();
        let attn_weights = (q.matmul(&k.t()?)? * scale)?;
        
        // 添加掩码 (Mask)
        // mask: [batch, 1, 1, seq_len]
        let attn_weights = attn_weights.broadcast_add(mask)?;
        
        let attn_weights = candle_nn::ops::softmax(&attn_weights, candle_core::D::Minus1)?;
        
        let attn_output = attn_weights.matmul(&v)?; // [batch, heads, seq, head_dim]
        
        let attn_output = attn_output.transpose(1, 2)?.reshape((batch, seq_len, hidden))?;
        
        // 输出投影 (Output projection)
        let attn_output = self.attention_output_dense.forward(&attn_output)?;
        let attn_output = match &self.attention_output_dense_bias {
            Some(b) => attn_output.broadcast_add(b)?,
            None => attn_output,
        };
        
        // 残差连接 + 归一化 (Residual + Norm)
        let attn_output = (attn_output + x)?;
        let attn_output = self.attention_output_norm.forward(&attn_output)?;
        
        // 前馈网络 (Feed Forward)
        // 中间层 (Intermediate)
        let intermediate = self.intermediate_dense.forward(&attn_output)?;
        let intermediate = match &self.intermediate_dense_bias {
            Some(b) => intermediate.broadcast_add(b)?,
            None => intermediate,
        };
        let intermediate = intermediate.gelu()?;
        
        // 输出层 (Output)
        let layer_output = self.output_dense.forward(&intermediate)?;
        let layer_output = match &self.output_dense_bias {
            Some(b) => layer_output.broadcast_add(b)?,
            None => layer_output,
        };
        
        // 残差连接 + 归一化 (Residual + Norm)
        let layer_output = (layer_output + attn_output)?;
        let layer_output = self.output_norm.forward(&layer_output)?;
        
        Ok(layer_output)
    }
}

impl QuantizedBertModel {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let mut file = std::fs::File::open(path)?;
        let content = gguf_file::Content::read(&mut file)?;
        let device = Device::Cpu;

        // 加载张量 (Load tensors)
        let mut layers = Vec::new();
        
        // 加载嵌入 (Load embeddings)
        let embeddings = content.tensor(&mut file, "token_embd.weight", &device)?.dequantize(&device)?;
        let position_embeddings = content.tensor(&mut file, "position_embd.weight", &device)?.dequantize(&device)?;
        let token_type_embeddings = if content.tensor_infos.contains_key("token_types.weight") {
                    let t = content.tensor(&mut file, "token_types.weight", &device)?.dequantize(&device)?;
                    if t.rank() == 1 {
                         Some(t.unsqueeze(0)?)
                    } else {
                         Some(t)
                    }
                } else {
                    None
                };
        
        let embd_norm_w = content.tensor(&mut file, "token_embd_norm.weight", &device)?.dequantize(&device)?;
        let embd_norm_b = content.tensor(&mut file, "token_embd_norm.bias", &device)?.dequantize(&device)?;
        let embeddings_norm = LayerNorm::new(embd_norm_w, embd_norm_b, 1e-5); 

        // 辅助函数：从元数据获取 u32
        let get_metadata_u32 = |key: &str, default: u32| -> u32 {
             match content.metadata.get(key) {
                 Some(candle_core::quantized::gguf_file::Value::U32(v)) => *v,
                 Some(candle_core::quantized::gguf_file::Value::U64(v)) => *v as u32,
                 Some(candle_core::quantized::gguf_file::Value::I32(v)) => *v as u32,
                 Some(candle_core::quantized::gguf_file::Value::I64(v)) => *v as u32,
                 _ => default,
             }
        };

        let block_count = get_metadata_u32("bert.block_count", 24) as usize;
        let head_count = get_metadata_u32("bert.attention.head_count", 16) as usize;

        println!("📊 Loading Quantized BERT: {} layers, {} heads", block_count, head_count);

        // 加载层 (Load layers)
        for i in 0..block_count {
            let prefix = format!("blk.{}", i);
            
            let attn_q = QMatMul::from_qtensor(content.tensor(&mut file, &format!("{}.attn_q.weight", prefix), &device)?)?;
            let attn_q_b = match content.tensor(&mut file, &format!("{}.attn_q.bias", prefix), &device) {
                Ok(t) => Some(t.dequantize(&device)?),
                Err(_) => None,
            };
            
            let attn_k = QMatMul::from_qtensor(content.tensor(&mut file, &format!("{}.attn_k.weight", prefix), &device)?)?;
            let attn_k_b = match content.tensor(&mut file, &format!("{}.attn_k.bias", prefix), &device) {
                Ok(t) => Some(t.dequantize(&device)?),
                Err(_) => None,
            };
            
            let attn_v = QMatMul::from_qtensor(content.tensor(&mut file, &format!("{}.attn_v.weight", prefix), &device)?)?;
            let attn_v_b = match content.tensor(&mut file, &format!("{}.attn_v.bias", prefix), &device) {
                Ok(t) => Some(t.dequantize(&device)?),
                Err(_) => None,
            };
            
            let attn_out = QMatMul::from_qtensor(content.tensor(&mut file, &format!("{}.attn_output.weight", prefix), &device)?)?;
            let attn_out_b = match content.tensor(&mut file, &format!("{}.attn_output.bias", prefix), &device) {
                Ok(t) => Some(t.dequantize(&device)?),
                Err(_) => None,
            };
            
            let attn_norm_w = content.tensor(&mut file, &format!("{}.attn_output_norm.weight", prefix), &device)?.dequantize(&device)?;
            let attn_norm_b = content.tensor(&mut file, &format!("{}.attn_output_norm.bias", prefix), &device)?.dequantize(&device)?;
            let attn_norm = LayerNorm::new(attn_norm_w, attn_norm_b, 1e-5);
            
            // FFN
            let ffn_up = QMatMul::from_qtensor(content.tensor(&mut file, &format!("{}.ffn_up.weight", prefix), &device)?)?;
            let ffn_up_b = match content.tensor(&mut file, &format!("{}.ffn_up.bias", prefix), &device) {
                Ok(t) => Some(t.dequantize(&device)?),
                Err(_) => None,
            };
            
            let ffn_down = QMatMul::from_qtensor(content.tensor(&mut file, &format!("{}.ffn_down.weight", prefix), &device)?)?;
            let ffn_down_b = match content.tensor(&mut file, &format!("{}.ffn_down.bias", prefix), &device) {
                Ok(t) => Some(t.dequantize(&device)?),
                Err(_) => None,
            };
            
            let layer_norm_w = content.tensor(&mut file, &format!("{}.layer_output_norm.weight", prefix), &device)?.dequantize(&device)?;
            let layer_norm_b = content.tensor(&mut file, &format!("{}.layer_output_norm.bias", prefix), &device)?.dequantize(&device)?;
            let layer_norm = LayerNorm::new(layer_norm_w, layer_norm_b, 1e-5);
            
            layers.push(QuantizedBertLayer {
                attention_self_query: attn_q,
                attention_self_query_bias: attn_q_b,
                attention_self_key: attn_k,
                attention_self_key_bias: attn_k_b,
                attention_self_value: attn_v,
                attention_self_value_bias: attn_v_b,
                attention_output_dense: attn_out,
                attention_output_dense_bias: attn_out_b,
                attention_output_norm: attn_norm,
                intermediate_dense: ffn_up,
                intermediate_dense_bias: ffn_up_b,
                output_dense: ffn_down,
                output_dense_bias: ffn_down_b,
                output_norm: layer_norm,
                num_heads: head_count,
            });
        }

        Ok(Self {
            embeddings,
            position_embeddings,
            token_type_embeddings,
            embeddings_norm,
            layers,
            device,
        })
    }

    pub fn hidden_size(&self) -> usize {
        self.embeddings.dim(1).unwrap_or(0)
    }

    pub fn forward(&self, input_ids: &Tensor, token_type_ids: Option<&Tensor>) -> Result<Tensor> {
        let (batch, seq_len) = input_ids.dims2()?;
        let input_ids_flat = input_ids.flatten_all()?;
        let token_emb = self.embeddings.index_select(&input_ids_flat, 0)?;
        let token_emb = token_emb.reshape((batch, seq_len, ()))?;

        let positions = Tensor::arange(0u32, seq_len as u32, &self.device)?;
        let positions = positions.broadcast_as((batch, seq_len))?;
        let positions_flat = positions.flatten_all()?;
        let pos_emb = self.position_embeddings.index_select(&positions_flat, 0)?;
        let pos_emb = pos_emb.reshape((batch, seq_len, ()))?;

        let type_emb = if let Some(ref types) = self.token_type_embeddings {
            if let Some(token_type_ids) = token_type_ids {
                let types_flat = token_type_ids.flatten_all()?;
                let t_emb = types.index_select(&types_flat, 0)?;
                t_emb.reshape((batch, seq_len, ()))?
            } else {
                Tensor::zeros_like(&token_emb)?
            }
        } else {
            Tensor::zeros_like(&token_emb)?
        };

        let mut embeddings = (token_emb + pos_emb)?;
        embeddings = (embeddings + type_emb)?;
        embeddings = self.embeddings_norm.forward(&embeddings)?;
        
        // 掩码计算 (Mask calculation)
        // 优化：手动构建 Mask Tensor，减少中间 Tensor 的创建
        // 使用 0 作为 BERT 的填充 token id。
        let (batch, seq_len) = input_ids.dims2()?;
        let mask = if matches!(input_ids.device(), Device::Cpu) {
            // 自定义算子实现：直接遍历 input_ids 生成 f32 mask
            let input_vec = input_ids.flatten_all()?.to_vec1::<u32>()?;
            let mask_vec: Vec<f32> = input_vec.into_iter()
                .map(|id| if id == 0 { -1e9 } else { 0.0 })
                .collect();
            Tensor::from_vec(mask_vec, (batch, 1, 1, seq_len), input_ids.device())?
        } else {
            // Fallback (GPU etc.)
            let mask = input_ids.eq(0u32)?.to_dtype(DType::F32)?;
            let mask = (mask * -1e9)?;
            mask.unsqueeze(1)?.unsqueeze(1)?
        };

        let mut hidden_states = embeddings;
        for layer in &self.layers {
            hidden_states = layer.forward(&hidden_states, &mask)?;
        }
        Ok(hidden_states)
    }
}
