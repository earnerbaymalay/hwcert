//! SMBIOS information

use crate::{FirmwareError, Result, SmbiosInfo};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmbiosInfo {
    pub id: Uuid,
    pub bios_vendor: String,
    pub bios_version: String,
    pub bios_release_date: String,
    pub bios_revision: String,
    pub system_manufacturer: String,
    pub system_product_name: String,
    pub system_version: String,
    pub system_serial: String,
    pub system_uuid: String,
    pub system_sku: String,
    pub system_family: String,
    pub baseboard_manufacturer: String,
    pub baseboard_product: String,
    pub baseboard_version: String,
    pub baseboard_serial: String,
    pub baseboard_asset_tag: String,
    pub chassis_manufacturer: String,
    pub chassis_type: String,
    pub chassis_version: String,
    pub chassis_serial: String,
    pub chassis_asset_tag: String,
    pub processor_count: u32,
    pub processor_manufacturer: String,
    pub processor_version: String,
    pub processor_frequency: u32,
}

pub fn gather_smbios_info() -> Result<SmbiosInfo> {
    #[cfg(target_os = "windows")]
    {
        // Use WMI (Win32_BIOS, Win32_ComputerSystem, etc.)
        Ok(SmbiosInfo {
            id: Uuid::new_v4(),
            bios_vendor: "Unknown".to_string(),
            bios_version: "Unknown".to_string(),
            bios_release_date: "Unknown".to_string(),
            bios_revision: "Unknown".to_string(),
            system_manufacturer: "Unknown".to_string(),
            system_product_name: "Unknown".to_string(),
            system_version: "Unknown".to_string(),
            system_serial: "Unknown".to_string(),
            system_uuid: "Unknown".to_string(),
            system_sku: "Unknown".to_string(),
            system_family: "Unknown".to_string(),
            baseboard_manufacturer: "Unknown".to_string(),
            baseboard_product: "Unknown".to_string(),
            baseboard_version: "Unknown".to_string(),
            baseboard_serial: "Unknown".to_string(),
            baseboard_asset_tag: "Unknown".to_string(),
            chassis_manufacturer: "Unknown".to_string(),
            chassis_type: "Unknown".to_string(),
            chassis_version: "Unknown".to_string(),
            chassis_serial: "Unknown".to_string(),
            chassis_asset_tag: "Unknown".to_string(),
            processor_count: 0,
            processor_manufacturer: "Unknown".to_string(),
            processor_version: "Unknown".to_string(),
            processor_frequency: 0,
        })
    }
    #[cfg(target_os = "linux")]
    {
        // Read from /sys/class/dmi/id/
        Ok(SmbiosInfo {
            id: Uuid::new_v4(),
            bios_vendor: "Unknown".to_string(),
            bios_version: "Unknown".to_string(),
            bios_release_date: "Unknown".to_string(),
            bios_revision: "Unknown".to_string(),
            system_manufacturer: "Unknown".to_string(),
            system_product_name: "Unknown".to_string(),
            system_version: "Unknown".to_string(),
            system_serial: "Unknown".to_string(),
            system_uuid: "Unknown".to_string(),
            system_sku: "Unknown".to_string(),
            system_family: "Unknown".to_string(),
            baseboard_manufacturer: "Unknown".to_string(),
            baseboard_product: "Unknown".to_string(),
            baseboard_version: "Unknown".to_string(),
            baseboard_serial: "Unknown".to_string(),
            baseboard_asset_tag: "Unknown".to_string(),
            chassis_manufacturer: "Unknown".to_string(),
            chassis_type: "Unknown".to_string(),
            chassis_version: "Unknown".to_string(),
            chassis_serial: "Unknown".to_string(),
            chassis_asset_tag: "Unknown".to_string(),
            processor_count: 0,
            processor_manufacturer: "Unknown".to_string(),
            processor_version: "Unknown".to_string(),
            processor_frequency: 0,
        })
    }
}
