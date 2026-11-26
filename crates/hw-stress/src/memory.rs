//! Memory stress testing

use crate::{MemoryStressConfig, MemoryStressResult, MemoryPattern, StressError, Result};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use rand::Rng;

pub struct MemoryStress {
    config: MemoryStressConfig,
    ops_counter: Arc<AtomicU64>,
    latency_sum: Arc<AtomicU64>,
    latency_count: Arc<AtomicU64>,
}

impl MemoryStress {
    pub fn new(config: MemoryStressConfig) -> Self {
        Self {
            config,
            ops_counter: Arc::new(AtomicU64::new(0)),
            latency_sum: Arc::new(AtomicU64::new(0)),
            latency_count: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn run(&self, duration: Duration) -> Result<MemoryStressResult> {
        let size_bytes = (self.config.size_gb * 1_073_741_824.0) as usize;
        let buffer = vec![0u8; size_bytes];
        let buffer_ptr = buffer.as_ptr() as usize;
        let pattern = self.config.pattern.clone();
        let stride = self.config.stride.max(1);
        let ops_counter = self.ops_counter.clone();
        let latency_sum = self.latency_sum.clone();
        let latency_count = self.latency_count.clone();

        let start = Instant::now();
        let duration_secs = duration.as_secs_f64();

        // Run memory access pattern
        match pattern {
            MemoryPattern::Sequential => {
                Self::sequential_access(buffer_ptr, size_bytes, stride, &ops_counter, &latency_sum, &latency_count, duration);
            }
            MemoryPattern::Random => {
                Self::random_access(buffer_ptr, size_bytes, &ops_counter, &latency_sum, &latency_count, duration);
            }
            MemoryPattern::Strided => {
                Self::strided_access(buffer_ptr, size_bytes, stride, &ops_counter, &latency_sum, &latency_count, duration);
            }
            MemoryPattern::WalkingOnes => {
                Self::walking_ones(buffer_ptr, size_bytes, &ops_counter, &latency_sum, &latency_count, duration);
            }
            MemoryPattern::WalkingZeros => {
                Self::walking_zeros(buffer_ptr, size_bytes, &ops_counter, &latency_sum, &latency_count, duration);
            }
        }

        let total_ops = ops_counter.load(Ordering::Relaxed);
        let bandwidth_gbps = (total_ops as f64 * 8.0) / (duration_secs * 1_000_000_000.0);
        let latency_ns = if latency_count.load(Ordering::Relaxed) > 0 {
            latency_sum.load(Ordering::Relaxed) as f64 / latency_count.load(Ordering::Relaxed) as f64
        } else {
            0.0
        };

        Ok(MemoryStressResult {
            duration_secs,
            bandwidth_gbps,
            latency_ns,
            errors: 0,
        })
    }

    fn sequential_access(
        ptr: usize,
        size: usize,
        stride: usize,
        ops: &Arc<AtomicU64>,
        latency_sum: &Arc<AtomicU64>,
        latency_count: &Arc<AtomicU64>,
        duration: Duration,
    ) {
        let start = Instant::now();
        let end_ptr = ptr + size;
        let mut current = ptr;
        let mut rng = rand::thread_rng();

        while start.elapsed() < duration {
            let access_start = Instant::now();
            unsafe {
                let _ = std::ptr::read_volatile(current as *const u8);
            }
            let latency = access_start.elapsed().as_nanos() as u64;

            ops.fetch_add(1, Ordering::Relaxed);
            latency_sum.fetch_add(latency, Ordering::Relaxed);
            latency_count.fetch_add(1, Ordering::Relaxed);

            current += stride;
            if current >= end_ptr {
                current = ptr;
            }
        }
    }

    fn random_access(
        ptr: usize,
        size: usize,
        ops: &Arc<AtomicU64>,
        latency_sum: &Arc<AtomicU64>,
        latency_count: &Arc<AtomicU64>,
        duration: Duration,
    ) {
        let start = Instant::now();
        let end_ptr = ptr + size;
        let mut rng = rand::thread_rng();

        while start.elapsed() < duration {
            let offset = rng.gen_range(0..size);
            let access_ptr = ptr + offset;
            let access_start = Instant::now();

            unsafe {
                let _ = std::ptr::read_volatile(access_ptr as *const u8);
            }
            let latency = access_start.elapsed().as_nanos() as u64;

            ops.fetch_add(1, Ordering::Relaxed);
            latency_sum.fetch_add(latency, Ordering::Relaxed);
            latency_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn strided_access(
        ptr: usize,
        size: usize,
        stride: usize,
        ops: &Arc<AtomicU64>,
        latency_sum: &Arc<AtomicU64>,
        latency_count: &Arc<AtomicU64>,
        duration: Duration,
    ) {
        let start = Instant::now();
        let end_ptr = ptr + size;
        let mut current = ptr;

        while start.elapsed() < duration {
            let access_start = Instant::now();
            unsafe {
                let _ = std::ptr::read_volatile(current as *const u8);
            }
            let latency = access_start.elapsed().as_nanos() as u64;

            ops.fetch_add(1, Ordering::Relaxed);
            latency_sum.fetch_add(latency, Ordering::Relaxed);
            latency_count.fetch_add(1, Ordering::Relaxed);

            current += stride;
            if current >= end_ptr {
                current = ptr;
            }
        }
    }

    fn walking_ones(
        ptr: usize,
        size: usize,
        ops: &Arc<AtomicU64>,
        latency_sum: &Arc<AtomicU64>,
        latency_count: &Arc<AtomicU64>,
        duration: Duration,
    ) {
        let start = Instant::now();
        let mut pattern = 0xFFu8;
        let mut current = ptr;
        let end_ptr = ptr + size;

        while start.elapsed() < duration {
            let access_start = Instant::now();
            unsafe {
                std::ptr::write_volatile(current as *mut u8, pattern);
                let _ = std::ptr::read_volatile(current as *const u8);
            }
            let latency = access_start.elapsed().as_nanos() as u64;

            ops.fetch_add(1, Ordering::Relaxed);
            latency_sum.fetch_add(latency, Ordering::Relaxed);
            latency_count.fetch_add(1, Ordering::Relaxed);

            pattern = pattern.rotate_left(1);
            current += 1;
            if current >= end_ptr {
                current = ptr;
            }
        }
    }

    fn walking_zeros(
        ptr: usize,
        size: usize,
        ops: &Arc<AtomicU64>,
        latency_sum: &Arc<AtomicU64>,
        latency_count: &Arc<AtomicU64>,
        duration: Duration,
    ) {
        let start = Instant::now();
        let mut pattern = 0x00u8;
        let mut current = ptr;
        let end_ptr = ptr + size;

        while start.elapsed() < duration {
            let access_start = Instant::now();
            unsafe {
                std::ptr::write_volatile(current as *mut u8, pattern);
                let _ = std::ptr::read_volatile(current as *const u8);
            }
            let latency = access_start.elapsed().as_nanos() as u64;

            ops.fetch_add(1, Ordering::Relaxed);
            latency_sum.fetch_add(latency, Ordering::Relaxed);
            latency_count.fetch_add(1, Ordering::Relaxed);

            pattern = pattern.rotate_right(1);
            current += 1;
            if current >= end_ptr {
                current = ptr;
            }
        }
    }
}