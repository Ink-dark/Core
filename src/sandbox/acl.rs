use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccessRight {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResourceType {
    FileSystem,
    Registry,
    Network,
    Process,
    Thread,
    Memory,
    WindowStation,
    Desktop,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub r#type: ResourceType,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlEntry {
    pub resource: Resource,
    pub right: AccessRight,
    pub inheritance: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlList {
    entries: Vec<AccessControlEntry>,
    default_right: AccessRight,
}

impl AccessControlList {
    pub fn new(default_right: AccessRight) -> Self {
        Self {
            entries: Vec::new(),
            default_right,
        }
    }

    pub fn with_entries(mut self, entries: Vec<AccessControlEntry>) -> Self {
        self.entries = entries;
        self
    }

    pub fn add_entry(&mut self, entry: AccessControlEntry) {
        self.entries.push(entry);
    }

    pub fn check_access(&self, resource: &Resource) -> AccessRight {
        // 查找匹配的访问控制条目
        for entry in &self.entries {
            if self.matches_resource(&entry.resource, resource) {
                return entry.right.clone();
            }
        }

        // 如果没有找到匹配的条目，使用默认权限
        self.default_right.clone()
    }

    fn matches_resource(&self, entry_resource: &Resource, requested_resource: &Resource) -> bool {
        // 检查资源类型是否匹配
        if entry_resource.r#type != requested_resource.r#type {
            return false;
        }

        // 检查路径是否匹配
        // 这里可以实现更复杂的路径匹配逻辑，例如通配符匹配
        entry_resource.path == requested_resource.path
    }

    pub fn entries(&self) -> &Vec<AccessControlEntry> {
        &self.entries
    }

    pub fn default_right(&self) -> &AccessRight {
        &self.default_right
    }
}

impl Default for AccessControlList {
    fn default() -> Self {
        Self::new(AccessRight::Deny)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub acl: AccessControlList,
    pub isolation_level: super::IsolationLevel,
    pub allowed_processes: Vec<String>,
    pub allowed_ports: Vec<u16>,
}

impl SecurityPolicy {
    pub fn new() -> Self {
        Self {
            acl: AccessControlList::default(),
            isolation_level: super::IsolationLevel::Medium,
            allowed_processes: Vec::new(),
            allowed_ports: Vec::new(),
        }
    }

    pub fn with_acl(mut self, acl: AccessControlList) -> Self {
        self.acl = acl;
        self
    }

    pub fn with_isolation_level(mut self, level: super::IsolationLevel) -> Self {
        self.isolation_level = level;
        self
    }

    pub fn with_allowed_processes(mut self, processes: Vec<String>) -> Self {
        self.allowed_processes = processes;
        self
    }

    pub fn with_allowed_ports(mut self, ports: Vec<u16>) -> Self {
        self.allowed_ports = ports;
        self
    }

    pub fn is_process_allowed(&self, process_name: &str) -> bool {
        self.allowed_processes.contains(&process_name.to_string())
    }

    pub fn is_port_allowed(&self, port: u16) -> bool {
        self.allowed_ports.contains(&port)
    }
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acl_default_deny() {
        let acl = AccessControlList::default();
        let resource = Resource {
            r#type: ResourceType::FileSystem,
            path: "C:\\Windows".to_string(),
        };
        let right = acl.check_access(&resource);
        assert_eq!(right, AccessRight::Deny);
    }

    #[test]
    fn test_acl_allow_specific_resource() {
        let mut acl = AccessControlList::new(AccessRight::Deny);
        let allowed_resource = Resource {
            r#type: ResourceType::FileSystem,
            path: "C:\\Users".to_string(),
        };
        acl.add_entry(AccessControlEntry {
            resource: allowed_resource.clone(),
            right: AccessRight::Allow,
            inheritance: false,
        });
        
        // 测试允许的资源
        let right = acl.check_access(&allowed_resource);
        assert_eq!(right, AccessRight::Allow);
        
        // 测试其他资源（应该被拒绝）
        let other_resource = Resource {
            r#type: ResourceType::FileSystem,
            path: "C:\\Windows".to_string(),
        };
        let right = acl.check_access(&other_resource);
        assert_eq!(right, AccessRight::Deny);
    }

    #[test]
    fn test_security_policy() {
        let policy = SecurityPolicy::new()
            .with_allowed_processes(vec!["notepad.exe".to_string(), "cmd.exe".to_string()])
            .with_allowed_ports(vec![80, 443]);
        
        assert!(policy.is_process_allowed("notepad.exe"));
        assert!(policy.is_process_allowed("cmd.exe"));
        assert!(!policy.is_process_allowed("malware.exe"));
        
        assert!(policy.is_port_allowed(80));
        assert!(policy.is_port_allowed(443));
        assert!(!policy.is_port_allowed(1337));
    }
}

