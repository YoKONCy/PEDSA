# PEDSA 优化运行脚本
# 针对本地 CPU 优化以获得最大推理速度
#
# 优化项：
# 1. target-cpu=native: 启用所有 CPU 特定的指令集（AVX2, FMA 等）
# 2. RAYON_NUM_THREADS: 自动检测（通常为逻辑核心数）已被证明是最快的
# 3. release profile: 标准优化

Write-Host "🚀 Compiling and Running PEDSA Main Program with Native CPU Optimizations..." -ForegroundColor Green

# 设置 RUSTFLAGS 以使用原生 CPU 特性（AVX2, FMA 等）
$env:RUSTFLAGS="-C target-cpu=native"

# 清除 RAYON_NUM_THREADS 以让 Rayon 自动检测最佳线程数
$env:RAYON_NUM_THREADS=$null

# 运行主程序
cargo run --release --bin PEDSA_Embedding

# 如果从资源管理器运行，暂停以查看结果
if ($Host.Name -eq "ConsoleHost") {
    Read-Host "Press Enter to exit..."
}
