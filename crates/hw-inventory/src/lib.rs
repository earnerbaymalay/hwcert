//! Hardware inventory crate
//! Provides comprehensive hardware information gathering

pub mod cpu;
pub mod gpu;
pub mod memory;
pub mod storage;
pub mod battery;
pub mod usb;
pub mod pci;
pub mod system;

use hw_inventory::cpu::CpuInfo;
use hw_inventory::gpu::GpuInfo;
use hw_inventory::memory::MemoryInfo;
use hw_inventory::storage::StorageInfo;
use hw_inventory::battery::BatteryInfo;
use hw_inventory::usb::UsbDevice;
use hw_inventory::pci::PciDevice;
use hw_inventory::system::SystemInfo;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Complete hardware inventory snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInventory {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub system: SystemInfo,
    pub cpu: CpuInfo,
    pub gpus: Vec<GpuInfo>,
    pub memory: MemoryInfo,
    pub storage: Vec<StorageInfo>,
    pub batteries: Vec<BatteryInfo>,
    pub usb_devices: Vec<UsbDevice>,
    pub pci_devices: Vec<PciDevice>,
}

/// Errors for hardware inventory
#[derive(Error, Debug)]
pub enum InventoryError {
    #[error("Sysinfo error: {0}")]
    Sysinfo(String),
    #[error("NVML error: {0}")]
    Nvml(String),
    #[error("SMART error: {0}")]
    Smart(String),
    #[error("USB error: {0}")]
    Usb(String),
    #[error("PCI error: {0}")]
    Pci(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, InventoryError>;

/// Gather complete hardware inventory
pub async fn gather_inventory() -> Result<HardwareInventory> {
    let system = system::gather_system_info()?;
    let cpu = cpu::gather_cpu_info()?;
    let gpus = gpu::gather_gpu_info()?;
    let memory = memory::gather_memory_info()?;
    let storage = storage::gather_storage_info()?;
    let batteries = battery::gather_battery_info()?;
    let usb_devices = usb::gather_usb_devices()?;
    let pci_devices = pci::gather_pci_devices()?;

    Ok(HardwareInventory {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        system,
        cpu,
        gpus,
        memory,
        storage,
        batteries,
        usb_devices,
        pci_devices,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gather_inventory() {
        let result = gather_inventory().await;
        assert!(result.is_ok());
    }
}