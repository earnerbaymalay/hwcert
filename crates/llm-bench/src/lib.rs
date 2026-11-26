//! LLM inference benchmarking crate
//! Provides benchmarking for llama.cpp, candle, and ONNX Runtime

pub mod candle;
pub mod llama_cpp;
pub mod metrics;
pub mod onnx;
pub mod runner;
pub mod tokenizer;

use chrono::{DateTime, Utc};
use llm_bench::metrics::{BenchMetrics, LatencyMetrics, TokenMetrics};
use llm_bench::runner::{BenchConfig, BenchResult, BenchRunner};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchConfig {
    pub model_path: String,
    pub backend: Backend,
    pub prompt: String,
    pub max_tokens: usize,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: usize,
    pub num_runs: usize,
    pub warmup_runs: usize,
    pub batch_size: usize,
    pub context_size: usize,
    pub gpu_layers: Option<usize>,
    pub threads: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Backend {
    LlamaCpp,
    Candle,
    OnnxRuntime,
}

/// Benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchResult {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub config: BenchConfig,
    pub token_metrics: TokenMetrics,
    pub latency_metrics: LatencyMetrics,
    pub memory_usage: MemoryUsage,
    pub gpu_usage: Option<GpuUsage>,
    pub passed: bool,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetrics {
    pub tokens_per_sec: f64,
    pub total_tokens: usize,
    pub prompt_tokens: usize,
    pub generation_tokens: usize,
    pub ttft_ms: f64, // Time to first token
    pub tpot_ms: f64, // Time per output token
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyMetrics {
    pub avg_ms: f64,
    pub p50_ms: f64,
    pub p90_ms: f64,
    pub p99_ms: f64,
    pub max_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsage {
    pub peak_mb: f64,
    pub avg_mb: f64,
    pub vram_peak_mb: Option<f64>,
    pub vram_avg_mb: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuUsage {
    pub avg_utilization: f32,
    pub peak_utilization: f32,
    pub avg_temp: Option<f32>,
    pub max_temp: Option<f32>,
    pub power_draw_avg: Option<f32>,
    pub power_draw_max: Option<f32>,
}

/// Errors for benchmarking
#[derive(Error, Debug)]
pub enum BenchError {
    #[error("Model load error: {0}")]
    ModelLoad(String),
    #[error("Inference error: {0}")]
    Inference(String),
    #[error("Tokenizer error: {0}")]
    Tokenizer(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Timeout")]
    Timeout,
}

pub type Result<T> = std::result::Result<T, BenchError>;

/// Run a benchmark
pub async fn run_benchmark(config: BenchConfig) -> Result<BenchResult> {
    let mut runner = BenchRunner::new(config);
    runner.run().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bench_config() {
        let config = BenchConfig {
            model_path: "model.gguf".to_string(),
            backend: Backend::LlamaCpp,
            prompt: "Hello".to_string(),
            max_tokens: 100,
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            num_runs: 5,
            warmup_runs: 1,
            batch_size: 1,
            context_size: 4096,
            gpu_layers: Some(20),
            threads: Some(8),
        };
        assert_eq!(config.max_tokens, 100);
    }
}
