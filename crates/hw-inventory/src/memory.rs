//! Memory information gathering

use crate::{MemoryInfo, InventoryError, Result};
use sysinfo::System;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub id: Uuid,
    pub total: u64,
    pub available: u64,
    pub used: u64,
    pub swap_total: u64,
    pub swap_used: u64,
    pub modules: Vec<MemoryModule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryModule {
    pub id: Uuid,
    pub size: u64,
    pub speed: u32,
    pub type_: String,
    pub manufacturer: String,
    pub part_number: String,
    pub serial_number: String,
    pub location: String,
}

pub fn gather_memory_info() -> Result<MemoryInfo> {
    let mut sys = System::new_all();
    sys.refresh_memory();
    sys.refresh_components();

    let modules = sys.memory_modules().iter().map(|m| MemoryModule {
        id: Uuid::new_v4(),
        size: m.size(),
        speed: m.speed() as u32,
        type_: m.memory_type().to_string(),
        manufacturer: m.manufacturer().to_string(),
        part_number: m.part_number().to_string(),
        serial_number: m.serial_number().to_string(),
        location: m.location().to_string(),
    }).collect();

    Ok(MemoryInfo {
        id: Uuid::new_v4(),
        total: sys.total_memory(),
        available: sys.available_memory(),
        used: sys.used_memory(),
        swap_total: sys.total_swap(),
        swap_used: sys.used_swap(),
        modules,
    })
}