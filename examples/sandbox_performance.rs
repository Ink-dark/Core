use moda_sandbox::{Sandbox, SandboxConfig, IsolationLevel, ResourceLimits, SecurityPolicy};

fn main() {
    println!("Testing sandbox performance...");

    // 测试基本沙箱创建性能
    let start = std::time::Instant::now();
    for i in 0..10 {
        let sandbox = Sandbox::new();
        let instance = sandbox.build();
        if let Ok(_inst) = instance {
            // 不实际运行任何命令，只测试创建性能
            println!("Created sandbox instance {}", i);
        }
    }
    let duration = start.elapsed();
    println!("Created 10 basic sandboxes in {:?}", duration);

    // 测试配置沙箱创建性能
    let config = SandboxConfig {
        namespace: moda_sandbox::NamespaceConfig::new().with_pid(true),
        isolation_level: IsolationLevel::High,
        resource_limits: ResourceLimits::new().with_max_memory(100).with_max_processes(5),
        security_policy: SecurityPolicy::default(),
    };

    let start = std::time::Instant::now();
    for i in 0..10 {
        let sandbox = Sandbox::new().with_config(config.clone());
        let instance = sandbox.build();
        if let Ok(_inst) = instance {
            println!("Created configured sandbox instance {}", i);
        }
    }
    let duration = start.elapsed();
    println!("Created 10 configured sandboxes in {:?}", duration);

    println!("Sandbox performance test completed.");
}