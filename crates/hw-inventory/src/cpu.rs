//! CPU information gathering

use crate::{CpuInfo, InventoryError, Result};
use serde::{Deserialize, Serialize};
use sysinfo::{CpuRefreshKind, RefreshKind, System};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub id: Uuid,
    pub name: String,
    pub brand: String,
    pub vendor_id: String,
    pub cores: usize,
    pub threads: usize,
    pub frequency_base: u64,
    pub frequency_max: u64,
    pub architecture: String,
    pub cache_l1: u64,
    pub cache_l2: u64,
    pub cache_l3: u64,
    pub temperature: Option<f32>,
    pub usage_percent: f32,
    pub features: Vec<String>,
}

pub fn gather_cpu_info() -> Result<CpuInfo> {
    let mut sys =
        System::new_with_specifics(RefreshKind::new().with_cpu(CpuRefreshKind::everything()));
    sys.refresh_cpu_all();

    let cpus = sys.cpus();
    let first_cpu = cpus.first();

    Ok(CpuInfo {
        id: Uuid::new_v4(),
        name: first_cpu.map(|c| c.name().to_string()).unwrap_or_default(),
        brand: first_cpu.map(|c| c.brand().to_string()).unwrap_or_default(),
        vendor_id: first_cpu
            .map(|c| c.vendor_id().to_string())
            .unwrap_or_default(),
        cores: sys.physical_core_count().unwrap_or(0),
        threads: cpus.len(),
        frequency_base: first_cpu.map(|c| c.frequency()).unwrap_or(0),
        frequency_max: first_cpu.map(|c| c.max_frequency()).unwrap_or(0),
        architecture: std::env::consts::ARCH.to_string(),
        cache_l1: 0,
        cache_l2: 0,
        cache_l3: 0,
        temperature: None,
        usage_percent: cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len().max(1) as f32,
        features: first_cpu
            .map(|c| c.cpu_features().iter().map(|s| s.to_string()).collect())
            .unwrap_or_default(),
    })
}
