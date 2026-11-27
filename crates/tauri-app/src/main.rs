// Prevents an extra console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            inventory::gather_inventory_cmd,
            stress::run_stress_test_cmd,
            bench::run_benchmark_cmd,
            firmware::gather_firmware_report_cmd,
            cert::generate_certificate_cmd,
            cert::verify_certificate_cmd,
            key::generate_key_pair_cmd,
            key::sign_data_cmd,
            key::verify_signature_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn cpu_count() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

mod inventory {
    use hw_inventory::{gather_inventory, HardwareInventory};

    #[tauri::command]
    pub async fn gather_inventory_cmd() -> Result<HardwareInventory, String> {
        gather_inventory().await.map_err(|e| e.to_string())
    }
}

mod stress {
    use hw_stress::{
        run_stress_test, CpuStressConfig, CpuWorkload, DiskPattern, DiskStressConfig,
        MemoryPattern, MemoryStressConfig, StressTestConfig, StressTestResults,
    };

    #[derive(serde::Deserialize)]
    pub struct StressConfig {
        pub cpu_threads: Option<usize>,
        pub duration_secs: u64,
        pub memory_gb: Option<f64>,
        pub disk_path: Option<String>,
        pub disk_gb: Option<f64>,
        pub cooldown_secs: u64,
        pub thermal_limit_celsius: Option<f32>,
    }

    #[tauri::command]
    pub async fn run_stress_test_cmd(config: StressConfig) -> Result<StressTestResults, String> {
        let threads = crate::cpu_count();
        let config = StressTestConfig {
            cpu: Some(CpuStressConfig {
                threads: config.cpu_threads.unwrap_or(threads),
                workload: CpuWorkload::Mixed,
                affinity: None,
            }),
            memory: config.memory_gb.map(|gb| MemoryStressConfig {
                size_gb: gb,
                pattern: MemoryPattern::Random,
                stride: 64,
            }),
            disk: config
                .disk_path
                .zip(config.disk_gb)
                .map(|(path, gb)| DiskStressConfig {
                    path,
                    size_gb: gb,
                    pattern: DiskPattern::Mixed,
                    block_size: 4096,
                    threads,
                    fsync: true,
                }),
            gpu: None,
            duration_secs: config.duration_secs,
            cooldown_secs: config.cooldown_secs,
            thermal_limit_celsius: config.thermal_limit_celsius,
        };

        run_stress_test(config).await.map_err(|e| e.to_string())
    }
}

mod bench {
    use llm_bench::{run_benchmark, Backend, BenchConfig, BenchResult};

    #[derive(serde::Deserialize)]
    pub struct BenchConfigInput {
        pub model_path: String,
        pub backend: String,
        pub prompt: String,
        pub max_tokens: usize,
        pub runs: usize,
        pub temperature: f32,
        pub gpu_layers: Option<usize>,
        pub threads: Option<usize>,
    }

    fn parse_backend(name: &str) -> Result<Backend, String> {
        match name {
            #[cfg(feature = "llama")]
            "llama.cpp" => Ok(Backend::LlamaCpp),
            "candle" => Ok(Backend::Candle),
            #[cfg(feature = "onnx")]
            "onnx" => Ok(Backend::OnnxRuntime),
            "llama.cpp" | "onnx" => Err(format!(
                "Backend '{name}' is not enabled in this build (rebuild with the matching feature)"
            )),
            other => Err(format!("Unknown backend '{other}'")),
        }
    }

    #[tauri::command]
    pub async fn run_benchmark_cmd(config: BenchConfigInput) -> Result<BenchResult, String> {
        let config = BenchConfig {
            backend: parse_backend(&config.backend)?,
            model_path: config.model_path,
            prompt: config.prompt,
            max_tokens: config.max_tokens,
            temperature: config.temperature,
            top_p: 0.9,
            top_k: 40,
            num_runs: config.runs,
            warmup_runs: 1,
            batch_size: 1,
            context_size: 4096,
            gpu_layers: config.gpu_layers,
            threads: config.threads,
        };

        run_benchmark(config).await.map_err(|e| e.to_string())
    }
}

mod firmware {
    use fw_check::{gather_firmware_report, FirmwareReport};

    #[tauri::command]
    pub async fn gather_firmware_report_cmd() -> Result<FirmwareReport, String> {
        gather_firmware_report().await.map_err(|e| e.to_string())
    }
}

mod cert {
    use cert_gen::{
        generate_firmware_verification, generate_hardware_attestation,
        generate_stress_test_certificate, verify_certificate, Certificate, KeyPair,
    };
    use serde_json::Value;

    #[derive(serde::Deserialize)]
    pub struct CertInput {
        pub cert_type: String,
        pub payload: Value,
        pub private_key_pkcs8: Vec<u8>,
    }

    #[tauri::command]
    pub async fn generate_certificate_cmd(input: CertInput) -> Result<Certificate, String> {
        let key_pair =
            KeyPair::from_pkcs8(&input.private_key_pkcs8).map_err(|e| e.to_string())?;

        let result = match input.cert_type.as_str() {
            "hardware" => generate_hardware_attestation(&input.payload, &key_pair).await,
            "firmware" => generate_firmware_verification(&input.payload, &key_pair).await,
            "stress_test" => generate_stress_test_certificate(&input.payload, &key_pair).await,
            other => return Err(format!("Invalid certificate type '{other}'")),
        };
        result.map_err(|e| e.to_string())
    }

    #[derive(serde::Deserialize)]
    pub struct VerifyInput {
        pub certificate: Certificate,
        pub public_key_hex: String,
    }

    #[tauri::command]
    pub async fn verify_certificate_cmd(input: VerifyInput) -> Result<bool, String> {
        verify_certificate(&input.certificate, &input.public_key_hex).map_err(|e| e.to_string())
    }
}

mod key {
    use cert_gen::{verify_signature, KeyPair};

    #[derive(serde::Deserialize)]
    pub struct GenKeyInput {
        pub seed: Option<String>,
    }

    /// Returns (raw 32-byte secret key, public key hex).
    #[tauri::command]
    pub async fn generate_key_pair_cmd(input: GenKeyInput) -> Result<(Vec<u8>, String), String> {
        let key_pair = match input.seed {
            Some(seed) => KeyPair::from_seed(blake3::hash(seed.as_bytes()).as_bytes()),
            None => KeyPair::generate(),
        };
        Ok((key_pair.to_pkcs8(), key_pair.verifying_key_bytes()))
    }

    #[derive(serde::Deserialize)]
    pub struct SignInput {
        pub private_key_pkcs8: Vec<u8>,
        pub data: String,
    }

    #[tauri::command]
    pub async fn sign_data_cmd(input: SignInput) -> Result<String, String> {
        let key_pair = KeyPair::from_pkcs8(&input.private_key_pkcs8).map_err(|e| e.to_string())?;
        Ok(hex::encode(key_pair.sign(input.data.as_bytes()).to_bytes()))
    }

    #[derive(serde::Deserialize)]
    pub struct VerifyInput {
        pub public_key_hex: String,
        pub data: String,
        pub signature_hex: String,
    }

    #[tauri::command]
    pub async fn verify_signature_cmd(input: VerifyInput) -> Result<bool, String> {
        let pubkey = hex::decode(&input.public_key_hex).map_err(|e| e.to_string())?;
        let signature = hex::decode(&input.signature_hex).map_err(|e| e.to_string())?;
        verify_signature(&pubkey, input.data.as_bytes(), &signature).map_err(|e| e.to_string())
    }
}
