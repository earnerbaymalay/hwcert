//! llama.cpp backend for LLM benchmarking
//! Requires `llama` feature flag (disabled by default due to cmake + VS build tools requirement)

#[cfg(feature = "llama")]
mod llama_cpp_impl {
    use crate::{BenchConfig, BenchError, GpuUsage, LatencyMetrics, MemoryUsage, Result, TokenMetrics};
    use llama_cpp_2::{ContextParams, LlamaContext, LlamaModel, ModelParams, SamplingParams};
    use std::time::{Duration, Instant};
    use uuid::Uuid;

    pub struct LlamaCppRunner {
        model: LlamaModel,
        context: LlamaContext,
        config: BenchConfig,
    }

    impl LlamaCppRunner {
        pub fn new(config: BenchConfig) -> Result<Self> {
            let model_params = ModelParams::default()
                .with_n_gpu_layers(config.gpu_layers.unwrap_or(0) as i32)
                .with_n_threads(config.threads.unwrap_or(4) as i32)
                .with_n_ctx(config.context_size as i32);

            let model = LlamaModel::load_from_file(&config.model_path, model_params)
                .map_err(|e| BenchError::ModelLoad(e.to_string()))?;

            let ctx_params = ContextParams::default()
                .with_n_threads(config.threads.unwrap_or(4) as i32);

            let context = model
                .new_context(&model, ctx_params)
                .map_err(|e| BenchError::ModelLoad(e.to_string()))?;

            Ok(Self { model, context, config })
        }

        pub async fn run_benchmark(&mut self) -> Result<BenchConfig> {
            // Warmup runs
            for _ in 0..self.config.warmup_runs {
                self.run_inference()?;
            }

            // Actual benchmark runs
            let mut all_latencies = Vec::new();
            let mut total_tokens = 0;
            let mut prompt_tokens = 0;
            let mut generation_tokens = 0;
            let mut ttft_times = Vec::new();

            for _ in 0..self.config.num_runs {
                let (latencies, tokens, prompt_tok, gen_tok, ttft) = self.run_inference()?;
                all_latencies.extend(latencies);
                total_tokens += tokens;
                prompt_tokens += prompt_tok;
                generation_tokens += gen_tok;
                ttft_times.push(ttft);
            }

            // Calculate metrics
            let token_metrics = TokenMetrics {
                tokens_per_sec: total_tokens as f64 / all_latencies.iter().sum::<f64>(),
                total_tokens,
                prompt_tokens,
                generation_tokens,
                ttft_ms: ttft_times.iter().sum::<f64>() / ttft_times.len() as f64 * 1000.0,
                tpot_ms: all_latencies.iter().sum::<f64>() / all_latencies.len() as f64 * 1000.0,
            };

            let mut sorted = all_latencies.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let latency_metrics = LatencyMetrics {
                avg_ms: all_latencies.iter().sum::<f64>() / all_latencies.len() as f64 * 1000.0,
                p50_ms: sorted[sorted.len() / 2] * 1000.0,
                p90_ms: sorted[sorted.len() * 9 / 10] * 1000.0,
                p99_ms: sorted[sorted.len() * 99 / 100] * 1000.0,
                max_ms: sorted.last().copied().unwrap_or(0.0) * 1000.0,
            }

            Ok(self.config.clone())
        }

        fn run_inference(&mut self) -> Result<(Vec<f64>, usize, usize, usize, f64)> {
            // Tokenize prompt
            let tokens = self.model.tokenize(&self.config.prompt, true, false)
                .map_err(|e| BenchError::Inference(e.to_string()))?;
            let prompt_tokens = tokens.len();

            // Setup sampling
            let sampling = SamplingParams::default()
                .with_temperature(self.config.temperature)
                .with_top_p(self.config.top_p)
                .with_top_k(self.config.top_k as i32);

            // Generate
            let start = Instant::now();
            let mut generated = Vec::new();
            let mut first_token_time = None;

            for _ in 0..self.config.max_tokens {
                let token = self.context.sample_token(&sampling)
                    .map_err(|e| BenchError::Inference(e.to_string()))?;

                if first_token_time.is_none() {
                    first_token_time = Some(start.elapsed());
                }

                generated.push(token);

                // Check for EOS
                if self.model.is_eog_token(token) {
                    break;
                }
            }

            let total_time = start.elapsed().as_secs_f64();
            let ttft = first_token_time.map(|t| t.as_secs_f64()).unwrap_or(total_time);
            let generation_tokens = generated.len();

            // Calculate per-token latencies (simplified)
            let latencies = vec![total_time / generated.len() as f64; generated.len()];

            Ok((latencies, generated.len(), prompt_tokens, generation_tokens, ttft))
        }
    }
}

#[cfg(feature = "llama")]
pub use llama_cpp_impl::LlamaCppRunner;

#[cfg(not(feature = "llama"))]
pub struct LlamaCppRunner;

#[cfg(not(feature = "llama"))]
impl LlamaCppRunner {
    pub fn new(_config: crate::BenchConfig) -> crate::Result<Self> {
        Err(crate::BenchError::Config(
            "llama.cpp backend requires 'llama' feature. Enable with --features llama".to_string()
        ))
    }
}