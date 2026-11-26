//! CPU stress testing

use crate::{CpuStressConfig, CpuStressResult, CpuWorkload, StressError, Result};
use rayon::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

pub struct CpuStress {
    config: CpuStressConfig,
    ops_counter: Arc<AtomicU64>,
    errors: Arc<AtomicU64>,
}

impl CpuStress {
    pub fn new(config: CpuStressConfig) -> Self {
        Self {
            config,
            ops_counter: Arc::new(AtomicU64::new(0)),
            errors: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn run(&self, duration: Duration) -> Result<CpuStressResult> {
        let threads = self.config.threads.max(1);
        let workload = self.config.workload.clone();
        let ops_counter = self.ops_counter.clone();
        let errors = self.errors.clone();

        let handles: Vec<_> = (0..threads)
            .map(|_| {
                let workload = workload.clone();
                let ops_counter = ops_counter.clone();
                let errors = errors.clone();
                std::thread::spawn(move || {
                    Self::worker_loop(workload, ops_counter, errors, duration)
                })
            })
            .collect();

        let start = Instant::now();
        for handle in handles {
            handle.join().map_err(|_| StressError::Thread("Thread panicked".to_string()))?;
        }
        let duration_secs = start.elapsed().as_secs_f64();

        let total_ops = ops_counter.load(Ordering::Relaxed);
        let ops_per_sec = total_ops as f64 / duration_secs;

        Ok(CpuStressResult {
            duration_secs,
            ops_per_sec,
            avg_temp: None,
            max_temp: None,
            throttled: false,
            errors: errors.load(Ordering::Relaxed),
        })
    }

    fn worker_loop(workload: CpuWorkload, ops_counter: Arc<AtomicU64>, errors: Arc<AtomicU64>, duration: Duration) {
        let start = Instant::now();
        while start.elapsed() < duration {
            match workload {
                CpuWorkload::MatrixMultiply => {
                    Self::matrix_multiply_work(&ops_counter);
                }
                CpuWorkload::PrimeSieving => {
                    Self::prime_sieve_work(&ops_counter);
                }
                CpuWorkload::HashComputation => {
                    Self::hash_work(&ops_counter);
                }
                CpuWorkload::CryptoOperations => {
                    Self::crypto_work(&ops_counter, &errors);
                }
                CpuWorkload::Mixed => {
                    Self::mixed_work(&ops_counter, &errors);
                }
            }
        }
    }

    fn matrix_multiply_work(ops_counter: &Arc<AtomicU64>) {
        // 64x64 matrix multiply
        let mut a = [[0.0f64; 64]; 64];
        let mut b = [[0.0f64; 64]; 64];
        let mut c = [[0.0f64; 64]; 64];

        for i in 0..64 {
            for j in 0..64 {
                a[i][j] = (i * j) as f64 * 0.01;
                b[i][j] = (i + j) as f64 * 0.01;
            }
        }

        for i in 0..64 {
            for j in 0..64 {
                let mut sum = 0.0;
                for k in 0..64 {
                    sum += a[i][k] * b[k][j];
                }
                c[i][j] = sum;
            }
        }

        ops_counter.fetch_add(64 * 64 * 64, Ordering::Relaxed);
    }

    fn prime_sieve_work(ops_counter: &Arc<AtomicU64>) {
        const LIMIT: usize = 100_000;
        let mut sieve = vec![true; LIMIT];
        sieve[0] = false;
        sieve[1] = false;

        for i in 2..=(LIMIT as f64).sqrt() as usize {
            if sieve[i] {
                for j in (i * i..LIMIT).step_by(i) {
                    sieve[j] = false;
                }
            }
        }

        let count = sieve.iter().filter(|&&x| x).count();
        ops_counter.fetch_add(count as u64, Ordering::Relaxed);
    }

    fn hash_work(ops_counter: &Arc<AtomicU64>) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        for i in 0..1_000_000 {
            i.hash(&mut hasher);
            hasher.finish();
        }
        ops_counter.fetch_add(1_000_000, Ordering::Relaxed);
    }

    fn crypto_work(ops_counter: &Arc<AtomicU64>, errors: &Arc<AtomicU64>) {
        // Simulate crypto operations
        for _ in 0..10_000 {
            let mut data = vec![0u8; 32];
            getrandom::getrandom(&mut data).unwrap_or_else(|_| {
                errors.fetch_add(1, Ordering::Relaxed);
                data.fill(0);
            });
        }
        ops_counter.fetch_add(10_000, Ordering::Relaxed);
    }

    fn mixed_work(ops_counter: &Arc<AtomicU64>, errors: &Arc<AtomicU64>) {
        Self::matrix_multiply_work(ops_counter);
        Self::prime_sieve_work(ops_counter);
        Self::hash_work(ops_counter);
        Self::crypto_work(ops_counter, errors);
    }
}