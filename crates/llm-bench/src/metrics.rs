//! Benchmark metrics

use crate::{GpuUsage, LatencyMetrics, MemoryUsage, TokenMetrics};

impl TokenMetrics {
    pub fn new() -> Self {
        Self {
            tokens_per_sec: 0.0,
            total_tokens: 0,
            prompt_tokens: 0,
            generation_tokens: 0,
            ttft_ms: 0.0,
            tpot_ms: 0.0,
        }
    }
}

impl LatencyMetrics {
    pub fn from_latencies(latencies: &[f64]) -> Self {
        let mut sorted = latencies.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let len = sorted.len();
        Self {
            avg_ms: latencies.iter().sum::<f64>() / len as f64 * 1000.0,
            p50_ms: sorted[len / 2] * 1000.0,
            p90_ms: sorted[len * 9 / 10] * 1000.0,
            p99_ms: sorted[len * 99 / 100] * 1000.0,
            max_ms: sorted.last().copied().unwrap_or(0.0) * 1000.0,
        }
    }
}

impl MemoryUsage {
    pub fn new() -> Self {
        Self {
            peak_mb: 0.0,
            avg_mb: 0.0,
            vram_peak_mb: None,
            vram_avg_mb: None,
        }
    }
}

impl GpuUsage {
    pub fn new() -> Self {
        Self {
            avg_utilization: 0.0,
            peak_utilization: 0.0,
            avg_temp: None,
            max_temp: None,
            power_draw_avg: None,
            power_draw_max: None,
        }
    }
}
