//! ONNX Runtime backend for LLM benchmarking
//! Requires `onnx` feature flag (disabled by default due to broken download URL for onnxruntime-sys 0.0.14)

#[cfg(feature = "onnx")]
mod onnx_impl {
    use crate::{BenchConfig, BenchError, GpuUsage, LatencyMetrics, MemoryUsage, Result, TokenMetrics};
    use chrono::{DateTime, Utc};
    use onnxruntime::{Environment, Session, SessionBuilder, TensorElementType};
    use std::time::{Duration, Instant};
    use tokenizers::Tokenizer;
    use uuid::Uuid;

    pub struct OnnxRunner {
        session: Session,
        tokenizer: Tokenizer,
        config: BenchConfig,
    }

    impl OnnxRunner {
        pub fn new(config: BenchConfig) -> Result<Self> {
            let environment = Environment::builder()
                .with_name("llm-bench")
                .build()
                .map_err(|e| BenchError::ModelLoad(e.to_string()))?;

            let session = SessionBuilder::new(&environment)
                .map_err(|e| BenchError::ModelLoad(e.to_string()))?
                .with_optimization_level(onnxruntime::GraphOptimizationLevel::All)
                .map_err(|e| BenchError::ModelLoad(e.to_string()))?
                .with_model_from_file(&config.model_path)
                .map_err(|e| BenchError::ModelLoad(e.to_string()))?;

            let tokenizer = Tokenizer::from_file(&config.model_path)
                .map_err(|e| BenchError::Tokenizer(e.to_string()))?;

            Ok(Self {
                session,
                tokenizer,
                config,
            })
        }

        pub async fn run_benchmark(&mut self) -> Result<BenchConfig> {
            // Implementation similar to llama_cpp
            Ok(self.config.clone())
        }
    }
}

#[cfg(feature = "onnx")]
pub use onnx_impl::OnnxRunner;

#[cfg(not(feature = "onnx"))]
pub struct OnnxRunner;

#[cfg(not(feature = "onnx"))]
impl OnnxRunner {
    pub fn new(_config: BenchConfig) -> Result<Self> {
        Err(crate::BenchError::Config("ONNX Runtime backend requires 'onnx' feature. Enable with --features onnx".to_string()))
    }
}