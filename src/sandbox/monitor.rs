use super::{Result, SandboxError, SandboxInstance};
use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::time::Duration;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ProcessStatus {
    Running,
    Exited(i32),
    Crashed,
    Suspended,
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub status: ProcessStatus,
    pub memory_usage: Option<u64>,
    pub cpu_usage: Option<f32>,
    pub start_time: std::time::Instant,
}

pub struct ProcessMonitor {
    processes: Arc<Mutex<HashMap<u32, ProcessInfo>>>,
    monitor_thread: Option<thread::JoinHandle<()>>,
    shutdown_flag: Arc<(Mutex<bool>, Condvar)>,
}

impl ProcessMonitor {
    pub fn new() -> Self {
        let processes = Arc::new(Mutex::new(HashMap::new()));
        let shutdown_flag = Arc::new((Mutex::new(false), Condvar::new()));
        
        Self {
            processes,
            monitor_thread: None,
            shutdown_flag,
        }
    }

    pub fn start(&mut self) {
        let processes = self.processes.clone();
        let shutdown_flag = self.shutdown_flag.clone();
        
        self.monitor_thread = Some(thread::spawn(move || {
            let (lock, cvar) = &*shutdown_flag;
            
            loop {
                // 检查是否需要关闭
                let shutdown = lock.lock().unwrap();
                if *shutdown {
                    break;
                }
                
                // 监控进程状态
                let mut processes = processes.lock().unwrap();
                // 实现具体的进程监控逻辑
                for (_, _process_info) in processes.iter_mut() {
                    // 检查进程是否仍在运行
                    // 注意：这里只是模拟，实际实现需要根据平台特定的方法检查进程状态
                    // 在实际实现中，我们会检查进程是否仍然存在
                    
                    // 更新内存和CPU使用情况（这里只是模拟）
                    // 实际实现需要查询进程的内存和CPU使用情况
                }
                
                // 模拟监控延迟
                drop(processes);
                
                // 等待一段时间或直到收到关闭信号
                let _ = cvar.wait_timeout(shutdown, Duration::from_secs(1)).unwrap();
            }
        }));
    }

    pub fn stop(&mut self) {
        // 发送关闭信号
        let (lock, cvar) = &*self.shutdown_flag;
        let mut shutdown = lock.lock().unwrap();
        *shutdown = true;
        cvar.notify_one();
        
        // 等待监控线程结束
        if let Some(thread) = self.monitor_thread.take() {
            thread.join().unwrap();
        }
    }

    pub fn add_process(&self, instance: &SandboxInstance) {
        let pid = instance.id();
        let process_info = ProcessInfo {
            pid,
            name: "unknown".to_string(), // 这里可以通过PID获取进程名称
            status: ProcessStatus::Running,
            memory_usage: None,
            cpu_usage: None,
            start_time: std::time::Instant::now(),
        };
        
        let mut processes = self.processes.lock().unwrap();
        processes.insert(pid, process_info);
    }

    pub fn remove_process(&self, pid: u32) {
        let mut processes = self.processes.lock().unwrap();
        processes.remove(&pid);
    }

    pub fn get_process_info(&self, pid: u32) -> Option<ProcessInfo> {
        let processes = self.processes.lock().unwrap();
        processes.get(&pid).cloned()
    }

    pub fn list_processes(&self) -> Vec<ProcessInfo> {
        let processes = self.processes.lock().unwrap();
        processes.values().cloned().collect()
    }

    pub fn check_process_status(&self, pid: u32) -> Result<ProcessStatus> {
        let processes = self.processes.lock().unwrap();
        match processes.get(&pid) {
            Some(info) => Ok(info.status.clone()),
            None => Err(SandboxError::ProcessError(format!("Process with PID {} not found", pid))),
        }
    }
}

impl Drop for ProcessMonitor {
    fn drop(&mut self) {
        self.stop();
    }
}

impl Default for ProcessMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_monitor() {
        let mut monitor = ProcessMonitor::new();
        
        // 测试监控启动和停止
        monitor.start();
        monitor.stop();
    }

    #[test]
    fn test_process_monitor_add_remove() {
        let monitor = ProcessMonitor::new();
        
        // 创建一个沙箱实例用于测试
        let sandbox = super::super::Sandbox::new();
        let instance = sandbox.build().unwrap();
        let pid = instance.id();
        
        // 添加进程到监控
        monitor.add_process(&instance);
        
        // 检查进程信息
        let process_info = monitor.get_process_info(pid);
        assert!(process_info.is_some());
        assert_eq!(process_info.unwrap().pid, pid);
        
        // 列出所有进程
        let processes = monitor.list_processes();
        assert!(!processes.is_empty());
        
        // 检查进程状态
        let status = monitor.check_process_status(pid);
        assert!(status.is_ok());
        assert_eq!(status.unwrap(), ProcessStatus::Running);
        
        // 移除进程
        monitor.remove_process(pid);
        let process_info = monitor.get_process_info(pid);
        assert!(process_info.is_none());
    }
}

