//! GPU information gathering

use crate::{InventoryError, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub id: Uuid,
    pub name: String,
    pub vendor: String,
    pub driver_version: String,
    pub vram_total: u64,
    pub vram_used: u64,
    pub temperature: Option<f32>,
    pub usage_percent: f32,
    pub power_draw: Option<f32>,
    pub clock_graphics: Option<u32>,
    pub clock_memory: Option<u32>,
    pub compute_capability: Option<String>,
    pub cuda_cores: Option<u32>,
}

pub fn gather_gpu_info() -> Result<Vec<GpuInfo>> {
    let mut gpus = Vec::new();

    // Try NVML for NVIDIA GPUs
    #[cfg(target_os = "windows")]
    {
        if let Ok(nvml_gpus) = gather_nvidia_gpus() {
            gpus.extend(nvml_gpus);
        }
    }

    // Try sysinfo for basic GPU info
    if gpus.is_empty() {
        if let Ok(basic_gpus) = gather_basic_gpus() {
            gpus.extend(basic_gpus);
        }
    }

    if gpus.is_empty() {
        return Err(InventoryError::Nvml("No GPUs detected".to_string()));
    }

    Ok(gpus)
}

fn gather_nvidia_gpus() -> Result<Vec<GpuInfo>> {
    #[cfg(feature = "nvml")]
    {
        use nvml_wrapper::Nvml;
        let nvml = Nvml::init().map_err(|e| InventoryError::Nvml(e.to_string()))?;
        let device_count = nvml
            .device_count()
            .map_err(|e| InventoryError::Nvml(e.to_string()))?;
        let mut gpus = Vec::new();

        for i in 0..device_count {
            let device = nvml
                .device_by_index(i)
                .map_err(|e| InventoryError::Nvml(e.to_string()))?;
            let name = device
                .name()
                .map_err(|e| InventoryError::Nvml(e.to_string()))?;
            let memory = device
                .memory_info()
                .map_err(|e| InventoryError::Nvml(e.to_string()))?;
            let temp = device
                .temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
                .ok();
            let utilization = device.utilization_rates().ok();

            gpus.push(GpuInfo {
                id: Uuid::new_v4(),
                name,
                vendor: "NVIDIA".to_string(),
                driver_version: nvml.sys_driver_version().unwrap_or_default(),
                vram_total: memory.total,
                vram_used: memory.used,
                temperature: temp.map(|t| t as f32),
                usage_percent: utilization.map(|u| u.gpu as f32).unwrap_or(0.0),
                power_draw: device.power_usage().ok().map(|p| p as f32 / 1000.0),
                clock_graphics: device
                    .clock_info(nvml_wrapper::enum_wrappers::device::Clock::Graphics)
                    .ok()
                    .map(|c| c as u32),
                clock_memory: device
                    .clock_info(nvml_wrapper::enum_wrappers::device::Clock::Memory)
                    .ok()
                    .map(|c| c as u32),
                compute_capability: None,
                cuda_cores: None,
            });
        }

        Ok(gpus)
    }
    #[cfg(not(feature = "nvml"))]
    {
        Err(InventoryError::Nvml("NVML feature not enabled".to_string()))
    }
}

fn gather_basic_gpus() -> Result<Vec<GpuInfo>> {
    // Fallback: use sysinfo or WMI on Windows
    Ok(vec![])
}
