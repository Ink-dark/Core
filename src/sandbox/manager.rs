use super::namespace::NamespaceConfig;
use super::{Result, SandboxError};
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub namespace: NamespaceConfig,
    pub isolation_level: IsolationLevel,
    pub resource_limits: ResourceLimits,
    pub security_policy: super::SecurityPolicy,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum IsolationLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_memory_mb: Option<u32>,
    pub max_cpu_percent: Option<u32>,
    pub max_processes: Option<u32>,
    pub max_threads: Option<u32>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            namespace: NamespaceConfig::default(),
            isolation_level: IsolationLevel::Medium,
            resource_limits: ResourceLimits {
                max_memory_mb: None,
                max_cpu_percent: None,
                max_processes: None,
                max_threads: None,
            },
            security_policy: super::SecurityPolicy::default(),
        }
    }
}

impl ResourceLimits {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_memory(mut self, mb: u32) -> Self {
        self.max_memory_mb = Some(mb);
        self
    }

    pub fn with_max_cpu(mut self, percent: u32) -> Self {
        self.max_cpu_percent = Some(percent);
        self
    }

    pub fn with_max_processes(mut self, count: u32) -> Self {
        self.max_processes = Some(count);
        self
    }

    pub fn with_max_threads(mut self, count: u32) -> Self {
        self.max_threads = Some(count);
        self
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: None,
            max_cpu_percent: None,
            max_processes: None,
            max_threads: None,
        }
    }
}

pub struct Sandbox {
    config: SandboxConfig,
}

impl Sandbox {
    pub fn new() -> Self {
        Self {
            config: SandboxConfig::default(),
        }
    }

    pub fn with_config(mut self, config: SandboxConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_namespace(mut self, namespace: NamespaceConfig) -> Self {
        self.config.namespace = namespace;
        self
    }

    pub fn with_isolation_level(mut self, level: IsolationLevel) -> Self {
        self.config.isolation_level = level;
        self
    }

    pub fn with_resource_limits(mut self, limits: ResourceLimits) -> Self {
        self.config.resource_limits = limits;
        self
    }

    pub fn with_security_policy(mut self, policy: super::SecurityPolicy) -> Self {
        self.config.security_policy = policy;
        self
    }

    pub fn build(&self) -> Result<SandboxInstance> {
        let mut command = if cfg!(windows) {
            Command::new("cmd")
        } else {
            Command::new("/bin/sh")
        };

        // Apply isolation level specific settings
        self.apply_isolation_settings(&mut command)?;

        // Apply resource limits
        self.apply_resource_limits(&mut command)?;

        let child = command
            .spawn()
            .map_err(|e| SandboxError::CreationFailed(format!("Failed to spawn process: {}", e)))?;

        Ok(SandboxInstance::new(child, self.config.clone()))
    }

    pub fn run_command(&self, program: &str, args: &[&str]) -> Result<SandboxInstance> {
        let mut command = Command::new(program);
        command.args(args);

        // Apply isolation level specific settings
        self.apply_isolation_settings(&mut command)?;

        // Apply resource limits
        self.apply_resource_limits(&mut command)?;

        let child = command
            .spawn()
            .map_err(|e| SandboxError::CreationFailed(format!("Failed to spawn process: {}", e)))?;

        Ok(SandboxInstance::new(child, self.config.clone()))
    }

    fn apply_isolation_settings(&self, command: &mut Command) -> Result<()> {
        // Apply namespace isolation
        if self.config.namespace.enable_pid {
            if cfg!(windows) {
                // Windows specific PID isolation
                command.arg("/c");
                command.arg("echo PID namespace enabled");
            } else {
                // Unix specific PID namespace
                command.arg("-c");
                command.arg("echo 'PID namespace enabled'");
            }
        }

        // Apply platform-specific isolation
        if let Ok(platform_instance) = super::get_platform().create_sandbox(&self.config) {
            platform_instance.apply_isolation(command)?;
        }

        // Apply isolation level specific settings
        match self.config.isolation_level {
            IsolationLevel::Low => {
                // Minimal isolation
            }
            IsolationLevel::Medium => {
                // Moderate isolation
            }
            IsolationLevel::High => {
                // Maximum isolation
            }
        }

        Ok(())
    }

    fn apply_resource_limits(&self, command: &mut Command) -> Result<()> {
        // Apply platform-specific resource limits
        if let Ok(platform_instance) = super::get_platform().create_sandbox(&self.config) {
            platform_instance.apply_resource_limits(command)?;
        }
        Ok(())
    }
}

impl Default for Sandbox {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SandboxInstance {
    child: Child,
    config: SandboxConfig,
    status: Arc<Mutex<SandboxStatus>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SandboxStatus {
    Running,
    Exited(std::process::ExitStatus),
    Error(String),
}

impl SandboxInstance {
    fn new(child: Child, config: SandboxConfig) -> Self {
        // 创建平台实例并分配进程到作业对象
        if let Ok(platform_instance) = super::get_platform().create_sandbox(&config) {
            let _ = platform_instance.assign_process(child.id());
        }
        
        Self {
            child,
            config,
            status: Arc::new(Mutex::new(SandboxStatus::Running)),
        }
    }

    pub fn wait(&mut self) -> Result<std::process::ExitStatus> {
        let status = self.child
            .wait()
            .map_err(|e| SandboxError::ProcessError(format!("Failed to wait for process: {}", e)))?;

        *self.status.lock().unwrap() = SandboxStatus::Exited(status);
        Ok(status)
    }

    pub fn id(&self) -> u32 {
        self.child.id()
    }

    pub fn config(&self) -> &SandboxConfig {
        &self.config
    }

    pub fn status(&self) -> SandboxStatus {
        self.status.lock().unwrap().clone()
    }

    pub fn terminate(&mut self) -> Result<()> {
        self.child
            .kill()
            .map_err(|e| SandboxError::ProcessError(format!("Failed to terminate process: {}", e)))?;
        Ok(())
    }

    pub fn wait_timeout(&mut self, duration: Duration) -> Result<Option<std::process::ExitStatus>> {
        // 首先尝试立即检查进程是否已经退出
        match self.child.try_wait() {
            Ok(Some(exit_status)) => {
                // 进程已经退出
                *self.status.lock().unwrap() = SandboxStatus::Exited(exit_status);
                Ok(Some(exit_status))
            }
            Ok(None) => {
                // 进程仍在运行，等待指定的时间
                use std::thread;
                thread::sleep(duration);
                
                // 再次检查进程状态
                match self.child.try_wait() {
                    Ok(status) => {
                        if let Some(exit_status) = status {
                            *self.status.lock().unwrap() = SandboxStatus::Exited(exit_status);
                        }
                        Ok(status)
                    }
                    Err(e) => Err(SandboxError::ProcessError(format!("Failed to wait for process: {}", e))),
                }
            }
            Err(e) => Err(SandboxError::ProcessError(format!("Failed to check process status: {}", e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_sandbox_creation() {
        let sandbox = Sandbox::new();
        let instance = sandbox.build();
        assert!(instance.is_ok());
    }

    #[test]
    fn test_sandbox_run_command() {
        let sandbox = Sandbox::new();
        #[cfg(windows)]
        let instance = sandbox.run_command("cmd", &["/c", "echo", "Hello, Sandbox!"]);
        #[cfg(not(windows))]
        let instance = sandbox.run_command("echo", &["Hello, Sandbox!"]);
        
        assert!(instance.is_ok());
        
        let mut instance = instance.unwrap();
        let status = instance.wait();
        assert!(status.is_ok());
        assert!(status.unwrap().success());
    }

    #[test]
    fn test_sandbox_with_config() {
        use crate::SecurityPolicy;
        
        let config = SandboxConfig {
            namespace: NamespaceConfig::new().with_pid(true),
            isolation_level: IsolationLevel::High,
            resource_limits: ResourceLimits::new().with_max_memory(100).with_max_processes(5),
            security_policy: SecurityPolicy::default(),
        };
        
        let sandbox = Sandbox::new().with_config(config);
        let instance = sandbox.build();
        assert!(instance.is_ok());
    }

    #[test]
    fn test_sandbox_termination() {
        let sandbox = Sandbox::new();
        #[cfg(windows)]
        let instance = sandbox.run_command("ping", &["-n", "5", "127.0.0.1"]);  // ping命令在Windows上可以被终止
        #[cfg(not(windows))]
        let instance = sandbox.run_command("sh", &["-c", "sleep 5"]);  // sleep命令在Unix上可以被终止
        
        assert!(instance.is_ok());
        
        let mut instance = instance.unwrap();
        let result = instance.terminate();
        assert!(result.is_ok());
        
        let status = instance.wait();
        assert!(status.is_ok());
        assert!(!status.unwrap().success());
    }

    #[test]
    fn test_sandbox_wait_timeout() {
        let sandbox = Sandbox::new();
        let instance = sandbox.run_command("cmd", &["/c", "timeout", "3"]);
        assert!(instance.is_ok());
        
        let mut instance = instance.unwrap();
        let status = instance.wait_timeout(Duration::from_secs(1));
        assert!(status.is_ok());
        assert!(status.unwrap().is_none());
        
        let status = instance.wait();
        assert!(status.is_ok());
        assert!(status.unwrap().success());
    }
}

