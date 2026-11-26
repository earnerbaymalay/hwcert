//! Battery information gathering

use crate::{BatteryInfo, InventoryError, Result};
use serde::{Deserialize, Serialize};
use sysinfo::System;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryInfo {
    pub id: Uuid,
    pub name: String,
    pub manufacturer: String,
    pub model: String,
    pub serial_number: String,
    pub chemistry: String,
    pub design_capacity: u32,
    pub full_charge_capacity: u32,
    pub current_capacity: u32,
    pub voltage: u32,
    pub charge_rate: i32,
    pub state: BatteryState,
    pub cycle_count: u32,
    pub design_voltage: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatteryState {
    Charging,
    Discharging,
    Full,
    Empty,
    Unknown,
}

pub fn gather_battery_info() -> Result<Vec<BatteryInfo>> {
    let mut sys = System::new_all();
    sys.refresh_battery();

    let mut batteries = Vec::new();

    // sysinfo doesn't expose detailed battery info directly
    // On Windows, we'd use WMI or WinRT APIs
    // For now, return empty vec

    Ok(batteries)
}
