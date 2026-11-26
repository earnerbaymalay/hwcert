//! Hardware stress testing crate
//! Provides CPU, GPU, memory, and disk stress testing

pub mod cpu;
pub mod disk;
pub mod gpu;
pub mod memory;
pub mod runner;
pub mod thermal;

use chrono::{DateTime, Utc};
use hw_stress::cpu::CpuStress;
use hw_stress::disk::DiskStress;
use hw_stress::gpu::GpuStress;
use hw_stress::memory::MemoryStress;
use hw_stress::runner::{StressConfig, StressResult, StressRunner};
use hw_stress::thermal::ThermalMonitor;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Stress test configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestConfig {
    pub cpu: Option<CpuStressConfig>,
    pub memory: Option<MemoryStressConfig>,
    pub disk: Option<DiskStressConfig>,
    pub gpu: Option<GpuStressConfig>,
    pub duration_secs: u64,
    pub cooldown_secs: u64,
    pub thermal_limit_celsius: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuStressConfig {
    pub threads: usize,
    pub workload: CpuWorkload,
    pub affinity: Option<Vec<usize>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CpuWorkload {
    MatrixMultiply,
    PrimeSieving,
    HashComputation,
    CryptoOperations,
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStressConfig {
    pub size_gb: f64,
    pub pattern: MemoryPattern,
    pub stride: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryPattern {
    Sequential,
    Random,
    Strided,
    WalkingOnes,
    WalkingZeros,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskStressConfig {
    pub path: String,
    pub size_gb: f64,
    pub pattern: DiskPattern,
    pub block_size: usize,
    pub threads: usize,
    pub fsync: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiskPattern {
    SequentialRead,
    SequentialWrite,
    RandomRead,
    RandomWrite,
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuStressConfig {
    pub device_index: usize,
    pub workload: GpuWorkload,
    pub intensity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GpuWorkload {
    MatrixMultiply,
    RayTracing,
    ComputeShader,
    TensorOperations,
    Mixed,
}

/// Stress test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestResults {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub config: StressTestConfig,
    pub cpu_result: Option<CpuStressResult>,
    pub memory_result: Option<MemoryStressResult>,
    pub disk_result: Option<DiskStressResult>,
    pub gpu_result: Option<GpuStressResult>,
    pub thermal_log: Vec<ThermalReading>,
    pub passed: bool,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuStressResult {
    pub duration_secs: f64,
    pub ops_per_sec: f64,
    pub avg_temp: Option<f32>,
    pub max_temp: Option<f32>,
    pub throttled: bool,
    pub errors: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStressResult {
    pub duration_secs: f64,
    pub bandwidth_gbps: f64,
    pub latency_ns: f64,
    pub errors: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskStressResult {
    pub duration_secs: f64,
    pub read_bandwidth_mbps: f64,
    pub write_bandwidth_mbps: f64,
    pub read_iops: f64,
    pub write_iops: f64,
    pub latency_avg_ms: f64,
    pub latency_p99_ms: f64,
    pub errors: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuStressResult {
    pub duration_secs: f64,
    pub ops_per_sec: f64,
    pub avg_temp: Option<f32>,
    pub max_temp: Option<f32>,
    pub power_draw_avg: Option<f32>,
    pub power_draw_max: Option<f32>,
    pub throttled: bool,
    pub errors: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalReading {
    pub timestamp: DateTime<Utc>,
    pub cpu_temp: Option<f32>,
    pub gpu_temp: Option<f32>,
    pub ambient_temp: Option<f32>,
}

/// Errors for stress testing
#[derive(Error, Debug)]
pub enum StressError {
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Hardware error: {0}")]
    Hardware(String),
    #[error("Thermal limit exceeded: {0}°C")]
    ThermalLimit(f32),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Thread error: {0}")]
    Thread(String),
    #[error("Timeout")]
    Timeout,
}

pub type Result<T> = std::result::Result<T, StressError>;

/// Run a complete stress test suite
pub async fn run_stress_test(config: StressTestConfig) -> Result<StressTestResults> {
    let mut runner = StressRunner::new(config.clone());
    runner.run().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stress_config() {
        let config = StressTestConfig {
            cpu: Some(CpuStressConfig {
                threads: 4,
                workload: CpuWorkload::Mixed,
                affinity: None,
            }),
            memory: None,
            disk: None,
            gpu: None,
            duration_secs: 10,
            cooldown_secs: 5,
            thermal_limit_celsius: Some(85.0),
        };
        assert_eq!(config.duration_secs, 10);
    }
}
