use super::{Result, SandboxConfig};
use std::process::Command;

pub trait SandboxPlatform {
    fn create_sandbox(&self, config: &SandboxConfig) -> Result<Box<dyn SandboxPlatformInstance>>;
}

pub trait SandboxPlatformInstance {
    fn apply_isolation(&self, command: &mut Command) -> Result<()>;
    fn apply_resource_limits(&self, command: &mut Command) -> Result<()>;
    fn assign_process(&self, process_id: u32) -> Result<()>;
    fn cleanup(&self) -> Result<()>;
}

#[cfg(windows)]
pub struct WindowsPlatform;

#[cfg(windows)]
impl SandboxPlatform for WindowsPlatform {
    fn create_sandbox(&self, config: &SandboxConfig) -> Result<Box<dyn SandboxPlatformInstance>> {
        Ok(Box::new(WindowsPlatformInstance::new(config)?))
    }
}

#[cfg(windows)]
pub struct WindowsPlatformInstance {
    job_object: super::windows::WindowsJobObject,
}

#[cfg(windows)]
impl WindowsPlatformInstance {
    pub fn new(config: &SandboxConfig) -> Result<Self> {
        let job_object = super::windows::WindowsJobObject::new()?;
        job_object.set_limits(config)?;
        Ok(Self { job_object })
    }
}

#[cfg(windows)]
impl SandboxPlatformInstance for WindowsPlatformInstance {
    fn apply_isolation(&self, _command: &mut Command) -> Result<()> {
        // 隔离设置在进程创建后应用
        Ok(())
    }

    fn apply_resource_limits(&self, _command: &mut Command) -> Result<()> {
        // 资源限制在作业对象中设置
        Ok(())
    }

    fn assign_process(&self, process_id: u32) -> Result<()> {
        self.job_object.assign_process(process_id)
    }

    fn cleanup(&self) -> Result<()> {
        // 作业对象会在Drop时自动清理
        Ok(())
    }
}

#[cfg(unix)]
pub struct UnixPlatform;

#[cfg(unix)]
impl SandboxPlatform for UnixPlatform {
    fn create_sandbox(&self, config: &SandboxConfig) -> Result<Box<dyn SandboxPlatformInstance>> {
        Ok(Box::new(UnixPlatformInstance::new(config)?))
    }
}

#[cfg(unix)]
pub struct UnixPlatformInstance {
    // Unix 特定的沙箱实现
}

#[cfg(unix)]
impl UnixPlatformInstance {
    pub fn new(_config: &SandboxConfig) -> Result<Self> {
        Ok(Self {})
    }
}

#[cfg(unix)]
impl SandboxPlatformInstance for UnixPlatformInstance {
    fn apply_isolation(&self, command: &mut Command) -> Result<()> {
        // 使用unshare系统调用来创建命名空间隔离
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            // 设置clone标志以创建新的命名空间
            unsafe {
                command.pre_exec(|| {
                    // 创建PID、网络、挂载和UTS命名空间
                    let flags = libc::CLONE_NEWPID | libc::CLONE_NEWNET | libc::CLONE_NEWNS | libc::CLONE_NEWUTS;
                    let result = libc::unshare(flags);
                    if result != 0 {
                        // 获取更具体的错误信息
                        let err = std::io::Error::last_os_error();
                        // 检查错误类型，可能是权限不足
                        if err.raw_os_error() == Some(libc::EPERM) {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::PermissionDenied,
                                format!("Insufficient privileges to create namespaces: {}", err)
                            ));
                        } else {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Failed to create namespaces: {} (errno: {})", err, result)
                            ));
                        }
                    }
                    Ok(())
                });
            }
        }
        Ok(())
    }

    fn apply_resource_limits(&self, command: &mut Command) -> Result<()> {
        // 设置Unix资源限制
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            unsafe {
                command.pre_exec(move || {
                    // 设置内存限制 (如果配置了的话)
                    let rlim = libc::rlimit {
                        rlim_cur: 1024 * 1024 * 100, // 100MB 软限制
                        rlim_max: 1024 * 1024 * 200, // 200MB 硬限制
                    };
                    if libc::setrlimit(libc::RLIMIT_AS, &rlim) != 0 {
                        return Err(std::io::Error::last_os_error());
                    }

                    // 设置进程数量限制
                    let proc_rlim = libc::rlimit {
                        rlim_cur: 10, // 软限制为10个进程
                        rlim_max: 20, // 硬限制为20个进程
                    };
                    if libc::setrlimit(libc::RLIMIT_NPROC, &proc_rlim) != 0 {
                        return Err(std::io::Error::last_os_error());
                    }

                    Ok(())
                });
            }
        }
        Ok(())
    }

    fn assign_process(&self, _process_id: u32) -> Result<()> {
        // 实现 Unix 特定的进程分配
        Ok(())
    }

    fn cleanup(&self) -> Result<()> {
        // 实现 Unix 特定的清理
        Ok(())
    }
}

#[cfg(not(any(windows, unix)))]
pub struct GenericPlatform;

#[cfg(not(any(windows, unix)))]
impl SandboxPlatform for GenericPlatform {
    fn create_sandbox(&self, _config: &SandboxConfig) -> Result<Box<dyn SandboxPlatformInstance>> {
        Ok(Box::new(GenericPlatformInstance {}))
    }
}

#[cfg(not(any(windows, unix)))]
pub struct GenericPlatformInstance;

#[cfg(not(any(windows, unix)))]
impl SandboxPlatformInstance for GenericPlatformInstance {
    fn apply_isolation(&self, _command: &mut Command) -> Result<()> {
        Ok(())
    }

    fn apply_resource_limits(&self, _command: &mut Command) -> Result<()> {
        Ok(())
    }

    fn assign_process(&self, _process_id: u32) -> Result<()> {
        Ok(())
    }

    fn cleanup(&self) -> Result<()> {
        Ok(())
    }
}

pub fn get_platform() -> Box<dyn SandboxPlatform> {
    #[cfg(windows)]
    return Box::new(WindowsPlatform {});
    
    #[cfg(unix)]
    return Box::new(UnixPlatform {});
    
    #[cfg(not(any(windows, unix)))]
    return Box::new(GenericPlatform {});
}
