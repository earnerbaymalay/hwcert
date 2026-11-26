//! Benchmark runner

use crate::{
    BenchConfig, BenchError, BenchResult, GpuUsage, LatencyMetrics, MemoryUsage, Result,
    TokenMetrics,
};
use candle::CandleRunner;
use chrono::{DateTime, Utc};
use llama_cpp::LlamaCppRunner;
use onnx::OnnxRunner;
use uuid::Uuid;

pub struct BenchRunner {
    config: BenchConfig,
}

impl BenchRunner {
    pub fn new(config: BenchConfig) -> Self {
        Self { config }
    }

    pub async fn run(&mut self) -> Result<BenchResult> {
        let result = match self.config.backend {
            crate::Backend::LlamaCpp => {
                let mut runner = LlamaCppRunner::new(self.config.clone())?;
                runner.run_benchmark().await?
            }
            crate::Backend::Candle => {
                let mut runner = CandleRunner::new(self.config.clone())?;
                runner.run_benchmark().await?
            }
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
