mod core;
mod ml;
mod data;
mod bench;


fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.contains(&"--v2".to_string()) || args.contains(&"--100m".to_string()) {
        bench::benchmarks::run_v2_benchmark(&args);
    } else if args.contains(&"--million".to_string()) || args.contains(&"--10m".to_string()) {
        bench::benchmarks::run_ten_million_test(10_000_000);
    } else if args.contains(&"--latency".to_string()) {
        bench::benchmark_latency::run_latency_benchmark();
    } else {
        println!("Tests are disabled for V2 native reconstruction.");
    }
}
