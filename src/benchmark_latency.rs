use pedsa_embedding::embedding::CandleModel;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Benchmark: Single Text Vectorization Latency (500 chars)");

    // 0. 设置线程数 (可选，如果未在环境变量中设置)
    // Candle 默认使用 Rayon 线程池，它会自动检测 CPU 核心数。
    // 但为了确保性能一致性，我们可以打印当前的线程配置。
    if let Ok(threads) = std::env::var("RAYON_NUM_THREADS") {
        println!("🧵 RAYON_NUM_THREADS: {}", threads);
    } else {
        println!("🧵 RAYON_NUM_THREADS: (Auto - defaults to logical cores)");
    }
    
    // 1. 加载模型
    let start_load = Instant::now();
    let model = CandleModel::new()?;
    let load_duration = start_load.elapsed();
    println!("✅ Model loaded in {:.2?}", load_duration);

    // 2. 准备 500 字符文本 (中英混合)
    let text = "Rust 是一种多范式、通用编程语言，强调性能、类型安全和并发性。它强制执行内存安全——这意味着所有引用都指向有效内存——而无需垃圾收集器或引用计数。Rust 项目最初由 Mozilla Research 的 Graydon Hoare 于 2006 年启动，并得到 Dave Herman、Brendan Eich 等人的贡献。自 2016 年以来，Rust 每年都在 Stack Overflow 开发者调查中被评为“最受喜爱的编程语言”。Rust is a multi-paradigm, general-purpose programming language that emphasizes performance, type safety, and concurrency. It enforces memory safety—meaning that all references point to valid memory—without requiring a garbage collector or reference counting present in other memory-safe languages. To enforce memory safety, Rust uses a borrow checker to track object lifetime and variable scope. Rust 语言的设计目标是提供高性能、安全性和并发性。它的语法类似于 C++，但在语义上更接近于 ML 家族语言。Rust 的核心概念是所有权（Ownership），它决定了谁可以访问和修改内存。通过所有权系统，Rust 可以在编译时检查内存错误，避免了运行时的段错误和数据竞争。Rust 还提供了丰富的标准库和工具链，使得开发变得更加高效和便捷。Rust 的社区非常活跃，拥有大量的开源库和框架，可以满足各种开发需求。无论是系统编程、Web 开发、嵌入式开发还是游戏开发，Rust 都能提供强大的支持。Rust 的未来充满了无限可能，它正在逐渐改变着编程世界的格局。让我们一起拥抱 Rust，开启高效编程的新篇章！";
    
    let char_count = text.chars().count();
    println!("📝 Text length: {} chars", char_count);

    // 3. 预热 (可选，用于将库加载到内存)
    println!("🔥 Warming up...");
    let _ = model.vectorize_weighted("Warm up", &[]);

    // 4. 基准测试循环
    let iterations = 10;
    println!("🚀 Running {} iterations...", iterations);
    
    let mut total_duration = std::time::Duration::new(0, 0);
    
    for i in 0..iterations {
        let start = Instant::now();
        let _vec = model.vectorize_weighted(text, &[]);
        let duration = start.elapsed();
        total_duration += duration;
        println!("   Iteration {}: {:.2?}", i + 1, duration);
    }

    let avg_duration = total_duration / iterations as u32;
    println!("\n📊 Average Latency: {:.2?}", avg_duration);
    println!("⚡ Throughput: {:.2} chars/sec", (char_count as f64 * iterations as f64) / total_duration.as_secs_f64());

    Ok(())
}
