//! System information gathering

use crate::{InventoryError, Result, SystemInfo};
use serde::{Deserialize, Serialize};
use sysinfo::{System, SystemExt};
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
    let mut sys = System::new_all();
    sys.refresh_system();

    Ok(SystemInfo {
        id: Uuid::new_v4(),
        hostname: sys.host_name().unwrap_or_default(),
        os_name: sys.name().unwrap_or_default(),
        os_version: sys.os_version().unwrap_or_default(),
        os_build: sys.long_os_version().unwrap_or_default(),
        kernel_version: sys.kernel_version().unwrap_or_default(),
        architecture: std::env::consts::ARCH.to_string(),
        uptime: sys.uptime(),
        boot_time: sys.boot_time(),
        total_memory: sys.total_memory(),
        used_memory: sys.used_memory(),
        total_swap: sys.total_swap(),
        used_swap: sys.used_swap(),
    })
}
