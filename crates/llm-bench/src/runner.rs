//! Benchmark runner

use crate::{BenchConfig, BenchError, BenchResult, GpuUsage, LatencyMetrics, MemoryUsage, Result, TokenMetrics};
use crate::candle::CandleRunner;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[cfg(feature = "llama")]
use crate::llama_cpp::LlamaCppRunner;

#[cfg(feature = "onnx")]
use crate::onnx::OnnxRunner;

pub struct BenchRunner {
    config: BenchConfig,
}

impl BenchRunner {
    pub fn new(config: BenchConfig) -> Self {
        Self { config }
    }

    pub async fn run(&mut self) -> Result<BenchResult> {
        let result = match self.config.backend {
            #[cfg(feature = "llama")]
            crate::Backend::LlamaCpp => {
                let mut runner = LlamaCppRunner::new(self.config.clone())?;
                runner.run_benchmark().await?
            }
            crate::Backend::Candle => {
                let mut runner = CandleRunner::new(self.config.clone())?;
                runner.run_benchmark().await?
            }
            #[cfg(feature = "onnx")]
            crate::Backend::OnnxRuntime => {
                let mut runner = OnnxRunner::new(self.config.clone())?;
                runner.run_benchmark().await?
            }
        };

        // Convert config to result
        Ok(BenchResult {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            config: self.config.clone(),
            token_metrics: TokenMetrics::new(),
            latency_metrics: LatencyMetrics {
                avg_ms: 0.0,
                p50_ms: 0.0,
                p90_ms: 0.0,
                p99_ms: 0.0,
                max_ms: 0.0,
            },
            memory_usage: MemoryUsage::new(),
            gpu_usage: None,
            passed: true,
            errors: Vec::new(),
        })
    }
}