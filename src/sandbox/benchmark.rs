use super::{Sandbox, SandboxConfig, IsolationLevel, ResourceLimits};
use std::time::Instant;

pub fn benchmark_sandbox_creation() {
    println!("Benchmarking sandbox creation...");
    
    let start = Instant::now();
    for _ in 0..100 {
        let sandbox = Sandbox::new();
        let _instance = sandbox.build();
    }
    let duration = start.elapsed();
    
    println!("Created 100 sandboxes in {:?}", duration);
    println!("Average time per sandbox: {:?}", duration / 100);
}

pub fn benchmark_sandbox_with_config() {
    println!("Benchmarking sandbox creation with config...");
    
    let config = SandboxConfig {
        namespace: super::NamespaceConfig::new().with_pid(true),
        isolation_level: IsolationLevel::High,
        resource_limits: ResourceLimits::new().with_max_memory(100).with_max_processes(5),
        security_policy: super::SecurityPolicy::default(),
    };
    
    let start = Instant::now();
    for _ in 0..100 {
        let sandbox = Sandbox::new().with_config(config.clone());
        let _instance = sandbox.build();
    }
    let duration = start.elapsed();
    
    println!("Created 100 configured sandboxes in {:?}", duration);
    println!("Average time per configured sandbox: {:?}", duration / 100);
}

pub fn run_all_benchmarks() {
    benchmark_sandbox_creation();
    benchmark_sandbox_with_config();
}