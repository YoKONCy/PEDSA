# GLiNER-X-Base (INT8 Quantized ONNX)

本目录存放 GLiNER-X-Base 的 ONNX 推理文件。  
`model.onnx` 因体积过大 (289 MB) **未包含在 Git 仓库中**，需手动导出。

## 导出方法

```bash
pip install gliner onnxruntime

python -c "
from gliner import GLiNER
from onnxruntime.quantization import quantize_dynamic, QuantType
import shutil, os

model = GLiNER.from_pretrained('knowledgator/gliner-x-base')
model.export_to_onnx('_tmp')
shutil.move('_tmp/model.onnx', 'model_fp32.onnx')
shutil.rmtree('_tmp')

# INT8 动态量化 (1152 MB → 289 MB, 精度基本不变)
quantize_dynamic('model_fp32.onnx', 'model.onnx', weight_type=QuantType.QInt8)
os.remove('model_fp32.onnx')
print('Done: model.onnx (INT8)')
"
```

## 文件清单

| 文件 | 大小 | 说明 | Git |
|------|------|------|-----|
| `model.onnx` | 289 MB | INT8 量化 ONNX | ❌ 需手动导出 |
| `tokenizer.json` | 15.6 MB | HuggingFace Tokenizer | ✅ |
| `gliner_config.json` | <1 KB | 模型配置 | ✅ |
| `LICENSE` | - | Apache-2.0 (Knowledgator) | ✅ |

## 原始模型

- **名称**: `knowledgator/gliner-x-base`
- **来源**: https://huggingface.co/knowledgator/gliner-x-base
- **许可**: Apache-2.0
- **修改**: ONNX 导出 + INT8 动态量化 (onnxruntime.quantization)
