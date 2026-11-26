//! Stress test runner

use crate::{StressTestConfig, StressTestResults, StressError, Result, ThermalMonitor, ThermalReading};
use hw_stress::cpu::CpuStress;
use hw_stress::memory::MemoryStress;
use hw_stress::disk::DiskStress;
use hw_stress::gpu::GpuStress;
use std::time::Duration;
use uuid::Uuid;
use chrono::{DateTime, Utc};

pub struct StressRunner {
    config: StressTestConfig,
    thermal_monitor: Option<ThermalMonitor>,
    thermal_readings: Vec<ThermalReading>,
}

impl StressRunner {
    pub fn new(config: StressTestConfig) -> Self {
        let thermal_monitor = config.thermal_limit_celsius.map(|limit| ThermalMonitor::new(Some(limit)));
        Self {
            config,
            thermal_monitor,
            thermal_readings: Vec::new(),
        }
    }

    pub async fn run(&mut self) -> Result<StressTestResults> {
        let start_time = Utc::now();
        let mut errors = Vec::new();

        // Start thermal monitoring
        if let Some(monitor) = &self.thermal_monitor {
            if let Err(e) = monitor.start() {
                errors.push(format!("Failed to start thermal monitor: {}", e));
            }
        }

        let mut cpu_result = None;
        let mut memory_result = None;
        let mut disk_result = None;
        let mut gpu_result = None;

        // Run CPU stress test
        if let Some(cpu_config) = &self.config.cpu {
            let stress = CpuStress::new(cpu_config.clone());
            match stress.run(Duration::from_secs(self.config.duration_secs)) {
                Ok(result) => cpu_result = Some(result),
                Err(e) => errors.push(format!("CPU stress failed: {}", e)),
            }
        }

        // Cooldown
        if self.config.cooldown_secs > 0 {
            tokio::time::sleep(Duration::from_secs(self.config.cooldown_secs)).await;
        }

        // Run memory stress test
        if let Some(mem_config) = &self.config.memory {
            let stress = MemoryStress::new(mem_config.clone());
            match stress.run(Duration::from_secs(self.config.duration_secs)) {
                Ok(result) => memory_result = Some(result),
                Err(e) => errors.push(format!("Memory stress failed: {}", e)),
            }
        }

        // Cooldown
        if self.config.cooldown_secs > 0 {
            tokio::time::sleep(Duration::from_secs(self.config.cooldown_secs)).await;
        }

        // Run disk stress test
        if let Some(disk_config) = &self.config.disk {
            let stress = DiskStress::new(disk_config.clone());
            match stress.run(Duration::from_secs(self.config.duration_secs)) {
                Ok(result) => disk_result = Some(result),
                Err(e) => errors.push(format!("Disk stress failed: {}", e)),
            }
        }

        // Cooldown
        if self.config.cooldown_secs > 0 {
            tokio::time::sleep(Duration::from_secs(self.config.cooldown_secs)).await;
        }

        // Run GPU stress test
        if let Some(gpu_config) = &self.config.gpu {
            let stress = GpuStress::new(gpu_config.clone());
            match stress.run(Duration::from_secs(self.config.duration_secs)) {
                Ok(result) => gpu_result = Some(result),
                Err(e) => errors.push(format!("GPU stress failed: {}", e)),
            }
        }

        // Stop thermal monitoring
        let thermal_readings = if let Some(monitor) = &self.thermal_monitor {
            monitor.stop()
        } else {
            Vec::new()
        };

        let passed = errors.is_empty() && self.check_results(&cpu_result, &memory_result, &disk_result, &gpu_result);

        Ok(StressTestResults {
            id: Uuid::new_v4(),
            timestamp: start_time,
            config: self.config.clone(),
            cpu_result,
            memory_result,
            disk_result,
            gpu_result,
            thermal_log: thermal_readings,
            passed,
            errors,
        })
    }

    fn check_results(
        &self,
        cpu: &Option<crate::CpuStressResult>,
        memory: &Option<crate::MemoryStressResult>,
        disk: &Option<crate::DiskStressResult>,
        gpu: &Option<crate::GpuStressResult>,
    ) -> bool {
        // Define pass criteria
        true
    }
}