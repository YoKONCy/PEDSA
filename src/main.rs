mod core;
mod ml;
mod data;
mod bench;
mod tests;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.contains(&"--v3".to_string()) || args.contains(&"--100m".to_string()) {
        bench::benchmarks::run_v3_benchmark(&args);
    } else if args.contains(&"--million".to_string()) || args.contains(&"--10m".to_string()) {
        bench::benchmarks::run_ten_million_test(10_000_000);
    } else if args.contains(&"--latency".to_string()) {
        bench::benchmark_latency::run_latency_benchmark();
    } else {
        tests::run_all_scenarios();
    }
}
