//! UEFI firmware information

use crate::{UefiInfo, FirmwareError, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UefiInfo {
    pub id: Uuid,
    pub vendor: String,
    pub version: String,
    pub release_date: String,
    pub revision: u32,
    pub secure_boot: bool,
    pub boot_mode: BootMode,
    pub variables: Vec<UefiVariable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BootMode {
    Uefi,
    Legacy,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UefiVariable {
    pub name: String,
    pub guid: String,
    pub attributes: u32,
    pub data_size: usize,
}

pub fn gather_uefi_info() -> Result<UefiInfo> {
    #[cfg(target_os = "windows")]
    {
        // Use Windows API (GetFirmwareEnvironmentVariableEx, etc.)
        Ok(UefiInfo {
            id: Uuid::new_v4(),
            vendor: "Unknown".to_string(),
            version: "Unknown".to_string(),
            release_date: "Unknown".to_string(),
            revision: 0,
            secure_boot: false,
            boot_mode: BootMode::Unknown,
            variables: Vec::new(),
        })
    }
    #[cfg(target_os = "linux")]
    {
        // Read from /sys/firmware/efi/
        Ok(UefiInfo {
            id: Uuid::new_v4(),
            vendor: "Unknown".to_string(),
            version: "Unknown".to_string(),
            release_date: "Unknown".to_string(),
            revision: 0,
            secure_boot: false,
            boot_mode: BootMode::Unknown,
            variables: Vec::new(),
        })
    }
}