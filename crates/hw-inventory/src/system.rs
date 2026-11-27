//! System information gathering

use crate::Result;
use serde::{Deserialize, Serialize};
use sysinfo::System;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub id: Uuid,
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub os_build: String,
    pub kernel_version: String,
    pub architecture: String,
    pub uptime: u64,
    pub boot_time: u64,
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
}

pub fn gather_system_info() -> Result<SystemInfo> {
    let mut sys = System::new();
    sys.refresh_memory();

    Ok(SystemInfo {
        id: Uuid::new_v4(),
        hostname: System::host_name().unwrap_or_default(),
        os_name: System::name().unwrap_or_default(),
        os_version: System::os_version().unwrap_or_default(),
        os_build: System::long_os_version().unwrap_or_default(),
        kernel_version: System::kernel_version().unwrap_or_default(),
        architecture: std::env::consts::ARCH.to_string(),
        uptime: System::uptime(),
        boot_time: System::boot_time(),
        total_memory: sys.total_memory(),
        used_memory: sys.used_memory(),
        total_swap: sys.total_swap(),
        used_swap: sys.used_swap(),
    })
}
