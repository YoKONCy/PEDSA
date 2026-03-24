use crate::ml::embedding::CandleModel;
use std::time::Instant;

/// 单文本向量化延迟基准测试 (500 字符, 10 次迭代)
pub fn run_latency_benchmark() {
    println!("Benchmark: Single Text Vectorization Latency");

    if let Ok(threads) = std::env::var("RAYON_NUM_THREADS") {
        println!("RAYON_NUM_THREADS: {}", threads);
    } else {
        println!("RAYON_NUM_THREADS: (Auto - defaults to logical cores)");
    }

    // 1. Load model
    let start_load = Instant::now();
    let model = match CandleModel::new() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Model load failed: {}", e);
            return;
        }
    };
    let load_duration = start_load.elapsed();
    println!("Model loaded in {:.2?}", load_duration);

    // 2. Prepare benchmark text
    let text = concat!(
        "Rust is a multi-paradigm, general-purpose programming language that emphasizes ",
        "performance, type safety, and concurrency. It enforces memory safety, meaning that ",
        "all references point to valid memory, without requiring a garbage collector or ",
        "reference counting present in other memory-safe languages. To enforce memory safety, ",
        "Rust uses a borrow checker to track object lifetime and variable scope. ",
        "Rust has been voted the most loved programming language in the Stack Overflow ",
        "Developer Survey every year since 2016. The Rust project was originally started by ",
        "Graydon Hoare at Mozilla Research in 2006, with contributions from Dave Herman, ",
        "Brendan Eich, and others. The core concept of Rust is ownership, which determines ",
        "who can access and modify memory. Through ownership, Rust can check memory errors ",
        "at compile time, avoiding segfaults and data races at runtime. Rust also provides a ",
        "rich standard library and toolchain, making development more efficient and convenient."
    );

    let char_count = text.chars().count();
    println!("Text length: {} chars", char_count);

    // 3. Warmup
    println!("Warming up...");
    let _ = model.vectorize_weighted("Warm up", &[]);

    // 4. Benchmark loop
    let iterations = 10;
    println!("Running {} iterations...", iterations);

    let mut total_duration = std::time::Duration::new(0, 0);

    for i in 0..iterations {
        let start = Instant::now();
        let _vec = model.vectorize_weighted(text, &[]);
        let duration = start.elapsed();
        total_duration += duration;
        println!("   Iteration {}: {:.2?}", i + 1, duration);
    }

    let avg_duration = total_duration / iterations as u32;
    println!("\nAverage Latency: {:.2?}", avg_duration);
    println!("Throughput: {:.2} chars/sec", (char_count as f64 * iterations as f64) / total_duration.as_secs_f64());
}
