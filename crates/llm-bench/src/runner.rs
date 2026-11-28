//! Benchmark runner

use crate::{BenchConfig, BenchResult, LatencyMetrics, MemoryUsage, Result, TokenMetrics};
use crate::candle::CandleRunner;
use chrono::Utc;
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
        if matches!(self.config.backend, crate::Backend::Ollama) {
            let run = crate::ollama::run(&self.config).await?;
            let mut config = self.config.clone();
            config.model_path = run.model; // record the model actually benchmarked
            config.endpoint = Some(run.endpoint);
            return Ok(BenchResult {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                config,
                passed: run.errors.is_empty(),
                token_metrics: run.token_metrics,
                latency_metrics: run.latency_metrics,
                // Ollama runs in its own process; its RSS/VRAM are not sampled here.
                memory_usage: MemoryUsage::new(),
                gpu_usage: None,
                errors: run.errors,
            });
        }

        // The other backends are still stubs: they load nothing and return the config.
        match self.config.backend {
            crate::Backend::Ollama => unreachable!("handled above"),
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

        Ok(BenchResult {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            config: self.config.clone(),
            token_metrics: TokenMetrics::new(),
            latency_metrics: LatencyMetrics::from_latencies_ms(&[]),
            memory_usage: MemoryUsage::new(),
            gpu_usage: None,
            // No metrics are measured for stub backends, so never report a pass.
            passed: false,
            errors: vec!["backend inference not implemented yet; use the ollama backend".to_string()],
        })
    }
}
