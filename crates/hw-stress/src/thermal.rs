//! Thermal monitoring

use crate::{Result, StressError, ThermalReading};
use chrono::Utc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct ThermalMonitor {
    running: Arc<AtomicBool>,
    thermal_limit: Option<f32>,
    readings: Arc<Mutex<Vec<ThermalReading>>>,
}

impl ThermalMonitor {
    pub fn new(thermal_limit: Option<f32>) -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            thermal_limit,
            readings: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn start(&self) -> Result<()> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Err(StressError::Hardware(
                "Thermal monitor already running".to_string(),
            ));
        }

        let running = self.running.clone();
        let thermal_limit = self.thermal_limit;
        let readings = self.readings.clone();

        std::thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                let cpu_temp = Self::read_cpu_temp();
                let gpu_temp = Self::read_gpu_temp();
                let ambient_temp = Self::read_ambient_temp();

                let reading = ThermalReading {
                    timestamp: Utc::now(),
                    cpu_temp,
                    gpu_temp,
                    ambient_temp,
                };

                let over_limit = thermal_limit.is_some_and(|limit| {
                    [cpu_temp, gpu_temp].into_iter().flatten().any(|t| t > limit)
                });
                if let Ok(mut log) = readings.lock() {
                    log.push(reading);
                }
                if over_limit {
                    running.store(false, Ordering::SeqCst);
                    break;
                }

                std::thread::sleep(Duration::from_secs(1));
            }
        });

        Ok(())
    }

    pub fn stop(&self) -> Vec<ThermalReading> {
        self.running.store(false, Ordering::SeqCst);
        self.readings
            .lock()
            .map(|mut log| std::mem::take(&mut *log))
            .unwrap_or_default()
    }

    fn read_cpu_temp() -> Option<f32> {
        #[cfg(target_os = "windows")]
        {
            // Use WMI or OpenHardwareMonitor
            None
        }
        #[cfg(target_os = "linux")]
        {
            // Read from /sys/class/thermal/
            None
        }
    }

    fn read_gpu_temp() -> Option<f32> {
        // GPU temperature comes from hw-inventory's NVML path; not sampled here yet.
        None
    }

    fn read_ambient_temp() -> Option<f32> {
        // Read from system sensors
        None
    }
}
