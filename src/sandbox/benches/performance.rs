use moda_sandbox::benchmark;

fn main() {
    println!("Running sandbox performance benchmarks...");
    benchmark::run_all_benchmarks();
    println!("Performance benchmarks completed.");
}