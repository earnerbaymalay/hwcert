//! Candle backend for LLM benchmarking

use crate::{BenchConfig, BenchError, Result};
use candle_core::Device;
use tokenizers::Tokenizer;

pub struct CandleRunner {
    #[allow(dead_code)]
    tokenizer: Tokenizer,
    #[allow(dead_code)]
    device: Device,
    config: BenchConfig,
}

impl CandleRunner {
    pub fn new(config: BenchConfig) -> Result<Self> {
        let device = Device::Cpu; // Use CPU for now
        let tokenizer = Tokenizer::from_file(&config.model_path)
            .map_err(|e| BenchError::Tokenizer(e.to_string()))?;

        // TODO: load real model weights (safetensors + config.json) via
        // candle_transformers::models::llama::Llama::load once model packaging is decided.
        // A zero-weight model was removed: it allocated a full-size model and benchmarked nothing.

        Ok(Self {
            tokenizer,
            device,
            config,
        })
    }

    pub async fn run_benchmark(&mut self) -> Result<BenchConfig> {
        // Implementation similar to llama_cpp
        Ok(self.config.clone())
    }
}
