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
    /// Summary of per-run latencies in milliseconds (nearest-rank percentiles; zeros if empty).
    pub fn from_latencies_ms(latencies: &[f64]) -> Self {
        if latencies.is_empty() {
            return Self { avg_ms: 0.0, p50_ms: 0.0, p90_ms: 0.0, p99_ms: 0.0, max_ms: 0.0 };
        }
        let mut sorted = latencies.to_vec();
        sorted.sort_by(|a, b| a.total_cmp(b));
        let pct = |p: f64| sorted[((p / 100.0 * sorted.len() as f64).ceil() as usize).clamp(1, sorted.len()) - 1];
        Self {
            avg_ms: sorted.iter().sum::<f64>() / sorted.len() as f64,
            p50_ms: pct(50.0),
            p90_ms: pct(90.0),
            p99_ms: pct(99.0),
            max_ms: sorted[sorted.len() - 1],
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latency_percentiles() {
        let l = LatencyMetrics::from_latencies_ms(&[40.0, 10.0, 30.0, 20.0]);
        assert_eq!((l.avg_ms, l.p50_ms, l.p90_ms, l.max_ms), (25.0, 20.0, 40.0, 40.0));
        assert_eq!(LatencyMetrics::from_latencies_ms(&[]).max_ms, 0.0);
    }
}
