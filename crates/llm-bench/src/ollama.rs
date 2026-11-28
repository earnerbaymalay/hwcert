//! Ollama backend: benchmark a model served by a local Ollama, using the
//! shared `ollama-client`. Endpoint and default model come from the
//! palexus-brain config unless the BenchConfig sets them.

use crate::{BenchConfig, BenchError, LatencyMetrics, Result, TokenMetrics};
use ollama_client::{Client, GenerateRequest, Metrics, Options};

/// What a run produced, before it becomes a BenchResult.
pub struct OllamaRun {
    pub model: String,
    pub endpoint: String,
    pub token_metrics: TokenMetrics,
    pub latency_metrics: LatencyMetrics,
    /// Model load time seen by the first (warmup) request, in ms.
    pub cold_load_ms: f64,
    pub errors: Vec<String>,
}

/// (endpoint, default model) from palexus-brain, falling back to Ollama defaults.
pub fn brain_defaults() -> (String, String) {
    match palexus_brain_schema::BrainConfig::load() {
        Ok(b) => (b.inference.ollama_url, b.inference.default_model),
        Err(_) => (ollama_client::DEFAULT_URL.to_string(), "qwen2.5-coder:1.5b".to_string()),
    }
}

pub async fn run(config: &BenchConfig) -> Result<OllamaRun> {
    let (brain_url, brain_model) = brain_defaults();
    let endpoint = config.endpoint.clone().unwrap_or(brain_url);
    let model = if config.model_path.trim().is_empty() { brain_model } else { config.model_path.clone() };
    let client = Client::new(&endpoint);

    let options = Options {
        temperature: Some(config.temperature),
        top_p: Some(config.top_p),
        top_k: Some(config.top_k as u32),
        max_tokens: Some(config.max_tokens as i32),
        context_size: Some(config.context_size as u32),
        // Fixed seed: runs differ only by hardware timing, not by sampling.
        seed: Some(42),
        stop: None,
    };
    let req = GenerateRequest::new(&model, &config.prompt).options(options);

    // Warmup also loads the model, so a missing model or dead server fails fast here.
    let mut cold_load_ms = 0.0;
    for i in 0..config.warmup_runs.max(1) {
        let r = client.generate(&req).await.map_err(|e| BenchError::Inference(format!("{model} @ {endpoint}: {e}")))?;
        if i == 0 {
            cold_load_ms = r.metrics.load_duration as f64 / 1e6;
        }
    }

    let mut runs: Vec<Metrics> = Vec::new();
    let mut errors = Vec::new();
    for i in 0..config.num_runs {
        match client.generate(&req).await {
            Ok(r) if r.metrics.eval_count == 0 => errors.push(format!("run {i}: model produced no tokens")),
            Ok(r) => runs.push(r.metrics),
            Err(e) => errors.push(format!("run {i}: {e}")),
        }
    }
    if runs.is_empty() && errors.is_empty() {
        errors.push("num_runs is 0; nothing measured".into());
    }

    Ok(OllamaRun {
        token_metrics: aggregate(&runs),
        latency_metrics: LatencyMetrics::from_latencies_ms(&runs.iter().map(Metrics::total_ms).collect::<Vec<_>>()),
        model,
        endpoint,
        cold_load_ms,
        errors,
    })
}

/// Mean throughput/latency across runs; token counts are totals.
fn aggregate(runs: &[Metrics]) -> TokenMetrics {
    let n = runs.len().max(1) as f64;
    let mean = |f: fn(&Metrics) -> f64| runs.iter().map(f).sum::<f64>() / n;
    let prompt_tokens: u64 = runs.iter().map(|m| m.prompt_eval_count).sum();
    let generation_tokens: u64 = runs.iter().map(|m| m.eval_count).sum();
    TokenMetrics {
        tokens_per_sec: mean(Metrics::tokens_per_sec),
        total_tokens: (prompt_tokens + generation_tokens) as usize,
        prompt_tokens: prompt_tokens as usize,
        generation_tokens: generation_tokens as usize,
        ttft_ms: mean(Metrics::ttft_ms),
        tpot_ms: mean(Metrics::tpot_ms),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregate_means_and_totals() {
        let m = |eval_count, eval_ms: u64, prompt_ms: u64| Metrics {
            total_duration: (eval_ms + prompt_ms) * 1_000_000,
            load_duration: 0,
            prompt_eval_count: 10,
            prompt_eval_duration: prompt_ms * 1_000_000,
            eval_count,
            eval_duration: eval_ms * 1_000_000,
        };
        let t = aggregate(&[m(20, 1000, 100), m(40, 1000, 300)]);
        assert_eq!(t.tokens_per_sec, 30.0); // mean of 20 and 40 tok/s
        assert_eq!(t.ttft_ms, 200.0);
        assert_eq!((t.prompt_tokens, t.generation_tokens), (20, 60));
        assert_eq!(aggregate(&[]).tokens_per_sec, 0.0);
    }
}
