pub mod manager;
pub mod namespace;
pub mod windows;
pub mod acl;
pub mod monitor;
pub mod platform;
pub mod benchmark;

pub use manager::{Sandbox, SandboxConfig, SandboxInstance, IsolationLevel, ResourceLimits};
pub use namespace::NamespaceConfig;
pub use acl::{AccessControlList, AccessControlEntry, AccessRight, Resource, ResourceType, SecurityPolicy};
pub use monitor::{ProcessMonitor, ProcessInfo, ProcessStatus};
pub use platform::{SandboxPlatform, SandboxPlatformInstance, get_platform};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SandboxError {
    #[error("Failed to create sandbox: {0}")]
    CreationFailed(String),
    #[error("Namespace isolation failed: {0}")]
    NamespaceError(String),
    #[error("Process spawn failed: {0}")]
    ProcessError(String),
    #[error("Windows API error: {0}")]
    WindowsApiError(String),
    #[error("Resource limit error: {0}")]
    ResourceLimitError(String),
}

pub type Result<T> = std::result::Result<T, SandboxError>;
