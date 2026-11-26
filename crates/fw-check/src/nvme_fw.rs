//! NVMe firmware information

use crate::{NvmeFirmware, FirmwareError, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NvmeFirmware {
    pub id: Uuid,
    pub device_path: String,
    pub model: String,
    pub serial: String,
    pub firmware_revision: String,
    pub namespace_count: u32,
    pub capacity_bytes: u64,
    pub pcie_link_speed: u32,
    pub pcie_link_width: u32,
    pub temperature: Option<u16>,
    pub health_percentage: Option<u8>,
    pub power_on_hours: u64,
    pub power_cycles: u64,
    pub unsafe_shutdowns: u64,
    pub media_errors: u64,
    pub num_err_log_entries: u64,
}

pub fn gather_nvme_firmware() -> Result<Vec<NvmeFirmware>> {
    #[cfg(target_os = "windows")]
    {
        // Use Windows Storage Management API or nvme-cli
        Ok(vec![])
    }
    #[cfg(target_os = "linux")]
    {
        // Use nvme-cli or /dev/nvme*
        Ok(vec![])
    }
}