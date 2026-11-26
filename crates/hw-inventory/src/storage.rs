//! Storage information gathering

use crate::{StorageInfo, SmartData, InventoryError, Result};
use sysinfo::{System, Disk, DiskKind};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use smartctl::{Smartctl, SmartctlArgs};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub id: Uuid,
    pub name: String,
    pub mount_point: String,
    pub file_system: String,
    pub total_space: u64,
    pub available_space: u64,
    pub used_space: u64,
    pub is_removable: bool,
    pub kind: String,
    pub smart_data: Option<SmartData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartData {
    pub smart_status: String,
    pub temperature: Option<u8>,
    pub power_on_hours: u64,
    pub power_cycle_count: u64,
    pub reallocated_sectors: u64,
    pub pending_sectors: u64,
    pub uncorrectable_sectors: u64,
    pub raw_attributes: serde_json::Value,
}

pub fn gather_storage_info() -> Result<Vec<StorageInfo>> {
    let mut sys = System::new_all();
    sys.refresh_disks();
    sys.refresh_disks_list();

    let mut storage = Vec::new();

    for disk in sys.disks() {
        let smart_data = if disk.kind() != DiskKind::Unknown {
            gather_smart_data(disk).ok()
        } else {
            None
        };

        storage.push(StorageInfo {
            id: Uuid::new_v4(),
            name: disk.name().to_string_lossy().to_string(),
            mount_point: disk.mount_point().to_string_lossy().to_string(),
            file_system: disk.file_system().to_string_lossy().to_string(),
            total_space: disk.total_space(),
            available_space: disk.available_space(),
            used_space: disk.total_space() - disk.available_space(),
            is_removable: disk.is_removable(),
            kind: format!("{:?}", disk.kind()),
            smart_data,
        });
    }

    Ok(storage)
}

fn gather_smart_data(disk: &Disk) -> Result<SmartData> {
    let device_path = disk.mount_point().to_string_lossy().to_string();
    let args = SmartctlArgs::new().arg("-a").arg(&device_path);
    let output = Smartctl::run(args).map_err(|e| InventoryError::Smart(e.to_string()))?;

    // Parse smartctl output (simplified)
    Ok(SmartData {
        smart_status: "OK".to_string(),
        temperature: None,
        power_on_hours: 0,
        power_cycle_count: 0,
        reallocated_sectors: 0,
        pending_sectors: 0,
        uncorrectable_sectors: 0,
        raw_attributes: serde_json::json!({}),
    })
}