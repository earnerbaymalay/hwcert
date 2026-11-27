//! GPU stress testing

use crate::{GpuStressConfig, GpuStressResult, Result};
#[cfg(not(feature = "gpu"))]
use crate::StressError;
use std::time::Duration;

pub struct GpuStress {
    #[allow(dead_code)] // read once a real CUDA/OpenCL backend replaces the placeholder
    config: GpuStressConfig,
}

impl GpuStress {
    pub fn new(config: GpuStressConfig) -> Self {
        Self { config }
    }

    #[cfg_attr(not(feature = "gpu"), allow(unused_variables))]
    pub fn run(&self, duration: Duration) -> Result<GpuStressResult> {
        #[cfg(feature = "gpu")]
        {
            self.run_gpu_stress(duration)
        }
        #[cfg(not(feature = "gpu"))]
        {
            Err(StressError::Config(
                "GPU stress testing not enabled".to_string(),
            ))
        }
    }

    #[cfg(feature = "gpu")]
    fn run_gpu_stress(&self, duration: Duration) -> Result<GpuStressResult> {
        // This would use CUDA/OpenCL/Metal APIs
        // For now, return a mock result
        let start = std::time::Instant::now();
        std::thread::sleep(duration);
        let duration_secs = start.elapsed().as_secs_f64();

        Ok(GpuStressResult {
            duration_secs,
            ops_per_sec: 0.0,
            avg_temp: None,
            max_temp: None,
            power_draw_avg: None,
            power_draw_max: None,
            throttled: false,
            errors: 0,
        })
    }
}
