use cert_gen::{generate_hardware_attestation, KeyPair};
use clap::{Parser, Subcommand};
use fw_check::gather_firmware_report;
use hw_inventory::gather_inventory;
use hw_stress::{run_stress_test, CpuStressConfig, CpuWorkload, StressTestConfig};
use llm_bench::{run_benchmark, Backend, BenchConfig};
use std::path::PathBuf;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "diagbox")]
#[command(about = "HWCert - IT Tech's Offline Diagnostic Suite", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Config file path
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Gather complete hardware inventory
    Inventory {
        /// Output format (json, yaml, table)
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Run stress tests
    Stress {
        /// Duration in seconds
        #[arg(short, long, default_value = "60")]
        duration: u64,

        /// CPU threads
        #[arg(long, default_value = "0")]
        cpu_threads: usize,

        /// Memory size in GB
        #[arg(long)]
        memory_gb: Option<f64>,

        /// Disk path
        #[arg(long)]
        disk_path: Option<String>,

        /// Disk size in GB
        #[arg(long)]
        disk_gb: Option<f64>,

        /// Cooldown between tests
        #[arg(long, default_value = "10")]
        cooldown: u64,

        /// Thermal limit in Celsius
        #[arg(long)]
        thermal_limit: Option<f32>,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Run LLM inference benchmark
    Bench {
        /// Model path
        #[arg(short, long)]
        model: String,

        /// Backend (llama.cpp, candle, onnx)
        #[arg(short, long, default_value = "llama.cpp")]
        backend: String,

        /// Prompt
        #[arg(short, long, default_value = "Explain quantum computing")]
        prompt: String,

        /// Max tokens
        #[arg(long, default_value = "100")]
        max_tokens: usize,

        /// Number of runs
        #[arg(long, default_value = "5")]
        runs: usize,

        /// Temperature
        #[arg(long, default_value = "0.7")]
        temperature: f32,

        /// GPU layers
        #[arg(long)]
        gpu_layers: Option<usize>,

        /// Threads
        #[arg(long)]
        threads: Option<usize>,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Check firmware
    Firmware {
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Check against database
        #[arg(long, default_value = "true")]
        check_db: bool,
    },

    /// Generate certificate
    Cert {
        /// Certificate type
        #[arg(short, long, value_enum)]
        cert_type: CertType,

        /// Input file (JSON)
        #[arg(short, long)]
        input: PathBuf,

        /// Private key file (PKCS#8)
        #[arg(short, long)]
        key: PathBuf,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Generate QR code
        #[arg(long, default_value = "true")]
        qr: bool,

        /// Generate PDF
        #[arg(long, default_value = "true")]
        pdf: bool,
    },

    /// Generate Ed25519 key pair
    GenKey {
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,

        /// Seed phrase (optional)
        #[arg(long)]
        seed: Option<String>,
    },

    /// Verify certificate
    Verify {
        /// Certificate file
        #[arg(short, long)]
        cert: PathBuf,

        /// Public key file (hex)
        #[arg(short, long)]
        pubkey: PathBuf,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum CertType {
    Hardware,
    Firmware,
    StressTest,
    Benchmark,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize logging
    let level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(level))
        .with(tracing_subscriber::fmt::layer())
        .init();

    match cli.command {
        Commands::Inventory { format, output } => {
            info!("Gathering hardware inventory...");
            let inventory = gather_inventory().await?;

            let output_str = match format.as_str() {
                "json" => serde_json::to_string_pretty(&inventory)?,
                "yaml" => serde_yaml::to_string(&inventory)?,
                _ => format!("{:#?}", inventory),
            };

            if let Some(path) = output {
                std::fs::write(path, output_str)?;
            } else {
                println!("{}", output_str);
            }
        }

        Commands::Stress {
            duration,
            cpu_threads,
            memory_gb,
            disk_path,
            disk_gb,
            cooldown,
            thermal_limit,
            output,
        } => {
            info!("Running stress tests...");

            let config = StressTestConfig {
                cpu: Some(CpuStressConfig {
                    threads: if cpu_threads > 0 {
                        cpu_threads
                    } else {
                        num_cpus::get()
                    },
                    workload: CpuWorkload::Mixed,
                    affinity: None,
                }),
                memory: memory_gb.map(|gb| hw_stress::MemoryStressConfig {
                    size_gb: gb,
                    pattern: hw_stress::MemoryPattern::Mixed,
                    stride: 64,
                }),
                disk: disk_path
                    .zip(disk_gb)
                    .map(|(path, gb)| hw_stress::DiskStressConfig {
                        path,
                        size_gb: gb,
                        pattern: hw_stress::DiskPattern::Mixed,
                        block_size: 4096,
                        threads: num_cpus::get(),
                        fsync: true,
                    }),
                gpu: None,
                duration_secs: duration,
                cooldown_secs: cooldown,
                thermal_limit_celsius: thermal_limit,
            };

            let result = run_stress_test(config).await?;

            let output_str = serde_json::to_string_pretty(&result)?;
            if let Some(path) = output {
                std::fs::write(path, output_str)?;
            } else {
                println!("{}", output_str);
            }
        }

        Commands::Bench {
            model,
            backend,
            prompt,
            max_tokens,
            runs,
            temperature,
            gpu_layers,
            threads,
            output,
        } => {
            info!("Running LLM benchmark...");

            let backend = match backend.as_str() {
                "llama.cpp" => Backend::LlamaCpp,
                "candle" => Backend::Candle,
                "onnx" => Backend::OnnxRuntime,
                _ => return Err("Invalid backend".into()),
            };

            let config = BenchConfig {
                model_path: model,
                backend,
                prompt,
                max_tokens,
                temperature,
                top_p: 0.9,
                top_k: 40,
                num_runs: runs,
                warmup_runs: 1,
                batch_size: 1,
                context_size: 4096,
                gpu_layers,
                threads,
            };

            let result = run_benchmark(config).await?;

            let output_str = serde_json::to_string_pretty(&result)?;
            if let Some(path) = output {
                std::fs::write(path, output_str)?;
            } else {
                println!("{}", output_str);
            }
        }

        Commands::Firmware { output, check_db } => {
            info!("Checking firmware...");
            let report = gather_firmware_report().await?;

            let output_str = serde_json::to_string_pretty(&report)?;
            if let Some(path) = output {
                std::fs::write(path, output_str)?;
            } else {
                println!("{}", output_str);
            }
        }

        Commands::Cert {
            cert_type,
            input,
            key,
            output,
            qr,
            pdf,
        } => {
            info!("Generating certificate...");

            let input_data = std::fs::read_to_string(&input)?;
            let payload: serde_json::Value = serde_json::from_str(&input_data)?;

            let key_data = std::fs::read(&key)?;
            let key_pair = KeyPair::from_pkcs8(&key_data)?;

            let cert = match cert_type {
                CertType::Hardware => {
                    cert_gen::generate_hardware_attestation(&payload, &key_pair).await?
                }
                CertType::Firmware => {
                    cert_gen::generate_firmware_verification(&payload, &key_pair).await?
                }
                CertType::StressTest => {
                    cert_gen::generate_stress_test_certificate(&payload, &key_pair).await?
                }
                CertType::Benchmark => {
                    // Similar to stress test
                    cert_gen::generate_stress_test_certificate(&payload, &key_pair).await?
                }
            };

            let output_str = serde_json::to_string_pretty(&cert)?;
            if let Some(path) = output {
                std::fs::write(path, output_str)?;
            } else {
                println!("{}", output_str);
            }
        }

        Commands::GenKey { output, seed } => {
            info!("Generating Ed25519 key pair...");

            let key_pair = if let Some(seed) = seed {
                let mut seed_bytes = [0u8; 32];
                let seed_hash = blake3::hash(seed.as_bytes());
                seed_bytes.copy_from_slice(&seed_hash.as_bytes()[..32]);
                KeyPair::from_seed(&seed_bytes)
            } else {
                KeyPair::generate()
            };

            let pkcs8 = key_pair.to_pkcs8();
            std::fs::write(&output, pkcs8)?;

            println!("Key pair generated: {}", output.display());
            println!("Public key: {}", key_pair.verifying_key_bytes());
        }

        Commands::Verify { cert, pubkey } => {
            info!("Verifying certificate...");

            let cert_data = std::fs::read_to_string(&cert)?;
            let cert: cert_gen::Certificate = serde_json::from_str(&cert_data)?;

            let pubkey_data = std::fs::read_to_string(&pubkey)?;
            let pubkey_bytes = hex::decode(pubkey_data.trim())?;

            let valid = cert_gen::verify_certificate(&cert, &hex::encode(pubkey_bytes))?;

            if valid {
                println!("✓ Certificate is valid");
            } else {
                println!("✗ Certificate verification failed");
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
