use super::{Result, SandboxError, SandboxConfig};
use std::ffi::OsString;
use std::os::windows::ffi::OsStrExt;
use std::process::Command;

#[cfg(windows)]
use winapi::um::winnt::HANDLE;
#[cfg(windows)]
use winapi::um::jobapi2::{CreateJobObjectW, SetInformationJobObject};
#[cfg(windows)]
use winapi::um::winnt::{
    JOBOBJECT_BASIC_LIMIT_INFORMATION, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JobObjectExtendedLimitInformation,
};

#[cfg(windows)]
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
#[cfg(windows)]
use winapi::um::errhandlingapi;
#[cfg(windows)]
use winapi::shared::minwindef::{DWORD, FALSE, LPVOID};

#[cfg(windows)]
pub struct WindowsJobObject {
    handle: HANDLE,
}

#[cfg(windows)]
impl WindowsJobObject {
    pub fn new() -> Result<Self> {
        let job_name = OsString::new();
        let job_name_ptr = job_name.as_os_str().encode_wide().chain(std::iter::once(0)).collect::<Vec<u16>>();
        
        let handle = unsafe {
            CreateJobObjectW(std::ptr::null_mut(), job_name_ptr.as_ptr())
        };
        
        if handle.is_null() || handle == INVALID_HANDLE_VALUE {
            let error_code = unsafe { winapi::um::errhandlingapi::GetLastError() };
            return Err(SandboxError::WindowsApiError(format!("Failed to create job object. Error code: {}", error_code)));
        }
        
        Ok(Self { handle })
    }
    
    pub fn set_limits(&self, config: &SandboxConfig) -> Result<()> {
        use winapi::um::winnt::{
            JOB_OBJECT_LIMIT_ACTIVE_PROCESS, JOB_OBJECT_LIMIT_JOB_MEMORY,
        };

        
        // 设置基本限制
        let mut basic_limit = unsafe { std::mem::zeroed::<JOBOBJECT_BASIC_LIMIT_INFORMATION>() };
        basic_limit.LimitFlags = 0;
        
        // 设置扩展限制
        let mut extended_limit = unsafe { std::mem::zeroed::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() };
        extended_limit.BasicLimitInformation = basic_limit;
        
        // 应用资源限制
        if let Some(max_memory) = config.resource_limits.max_memory_mb {
            // 设置JobMemoryLimit
            extended_limit.JobMemoryLimit = (max_memory * 1024 * 1024) as usize;
            extended_limit.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_JOB_MEMORY;
        }
        
        if let Some(max_processes) = config.resource_limits.max_processes {
            extended_limit.BasicLimitInformation.ActiveProcessLimit = max_processes as DWORD;
            extended_limit.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_ACTIVE_PROCESS;
        }
        
        // 设置作业对象信息
        let result = unsafe {
            SetInformationJobObject(
                self.handle,
                JobObjectExtendedLimitInformation,
                &mut extended_limit as *mut _ as LPVOID,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as DWORD
            )
        };
        
        if result == 0 {
            return Err(SandboxError::WindowsApiError("Failed to set job object limits".to_string()));
        }
        
        Ok(())
    }
    
    pub fn assign_process(&self, process_id: u32) -> Result<()> {
        use winapi::um::processthreadsapi::OpenProcess;
        use winapi::um::winnt::{PROCESS_SET_QUOTA, PROCESS_TERMINATE, PROCESS_SUSPEND_RESUME};
        let process_handle = unsafe {
            OpenProcess(
                PROCESS_SET_QUOTA | PROCESS_TERMINATE | PROCESS_SUSPEND_RESUME,
                FALSE,
                process_id as DWORD
            )
        };
        
        if process_handle == INVALID_HANDLE_VALUE {
            let error_code = unsafe { winapi::um::errhandlingapi::GetLastError() };
            return Err(SandboxError::WindowsApiError(format!("Failed to open process. Error code: {}", error_code)));
        }
        
        // 将进程分配到作业对象
        use winapi::um::jobapi2::AssignProcessToJobObject;
        let result = unsafe {
            AssignProcessToJobObject(self.handle, process_handle)
        };
        
        // 确保句柄被关闭，无论AssignProcessToJobObject是否成功
        unsafe {
            CloseHandle(process_handle);
        }
        
        if result == 0 {
            let error_code = unsafe { winapi::um::errhandlingapi::GetLastError() };
            return Err(SandboxError::WindowsApiError(format!("Failed to assign process to job object. Error code: {}", error_code)));
        }
        
        Ok(())
    }
    
    pub fn handle(&self) -> HANDLE {
        self.handle
    }
}

#[cfg(windows)]
impl Drop for WindowsJobObject {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.handle);
        }
    }
}

#[cfg(windows)]
pub fn apply_windows_isolation(_command: &mut Command, _config: &SandboxConfig) -> Result<()> {
    Ok(())
}

#[cfg(not(windows))]
pub fn apply_windows_isolation(_command: &mut Command, _config: &SandboxConfig) -> Result<()> {
    Ok(())
}
