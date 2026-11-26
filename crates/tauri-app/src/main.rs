use hw_inventory::gather_inventory;
use hw_stress::{run_stress_test, StressTestConfig, CpuStressConfig, CpuWorkload};
use llm_bench::{run_benchmark, BenchConfig, Backend};
use fw_check::gather_firmware_report;
use cert_gen::{generate_hardware_attestation, KeyPair};
use tauri::Manager;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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

mod inventory {
    use hw_inventory::{gather_inventory, HardwareInventory};
    use tauri::State;
    use std::sync::Mutex;

    #[tauri::command]
    pub async fn gather_inventory_cmd() -> Result<HardwareInventory, String> {
        gather_inventory().await.map_err(|e| e.to_string())
    }
}

mod stress {
    use hw_stress::{run_stress_test, StressTestConfig, CpuStressConfig, CpuWorkload, MemoryStressConfig, MemoryPattern, DiskStressConfig, DiskPattern};
    use tauri::State;
    use std::sync::Mutex;

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
    pub async fn run_stress_test_cmd(config: StressConfig) -> Result<hw_stress::StressTestResults, String> {
        let config = StressTestConfig {
            cpu: Some(CpuStressConfig {
                threads: config.cpu_threads.unwrap_or(num_cpus::get()),
                workload: CpuWorkload::Mixed,
                affinity: None,
            }),
            memory: config.memory_gb.map(|gb| MemoryStressConfig {
                size_gb: gb,
                pattern: MemoryPattern::Mixed,
                stride: 64,
            }),
            disk: config.disk_path.zip(config.disk_gb).map(|(path, gb)| DiskStressConfig {
                path,
                size_gb: gb,
                pattern: DiskPattern::Mixed,
                block_size: 4096,
                threads: num_cpus::get(),
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
    use llm_bench::{run_benchmark, BenchConfig, Backend};
    use tauri::State;

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

    #[tauri::command]
    pub async fn run_benchmark_cmd(config: BenchConfigInput) -> Result<llm_bench::BenchResult, String> {
        let backend = match config.backend.as_str() {
            "llama.cpp" => Backend::LlamaCpp,
            "candle" => Backend::Candle,
            "onnx" => Backend::OnnxRuntime,
            _ => return Err("Invalid backend".to_string()),
        };

        let config = BenchConfig {
            model_path: config.model_path,
            backend,
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
    use fw_check::gather_firmware_report;

    #[tauri::command]
    pub async fn gather_firmware_report_cmd() -> Result<fw_check::FirmwareReport, String> {
        gather_firmware_report().await.map_err(|e| e.to_string())
    }
}

mod cert {
    use cert_gen::{generate_hardware_attestation, verify_certificate, Certificate};
    use serde_json::Value;

    #[derive(serde::Deserialize)]
    pub struct CertInput {
        pub cert_type: String,
        pub payload: Value,
        pub private_key_pkcs8: Vec<u8>,
    }

    #[tauri::command]
    pub async fn generate_certificate_cmd(input: CertInput) -> Result<Certificate, String> {
        let key_pair = crate::cert_gen::KeyPair::from_pkcs8(&input.private_key_pkcs8)
            .map_err(|e| e.to_string())?;

        match input.cert_type.as_str() {
            "hardware" => generate_hardware_attestation(&input.payload, &key_pair).await,
            "firmware" => cert_gen::generate_firmware_verification(&input.payload, &key_pair).await,
            "stress_test" => cert_gen::generate_stress_test_certificate(&input.payload, &key_pair).await,
            _ => Err("Invalid certificate type".to_string()),
        }.map_err(|e| e.to_string())
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
    use crate::cert_gen::KeyPair;

    #[derive(serde::Deserialize)]
    pub struct GenKeyInput {
        pub seed: Option<String>,
    }

    #[tauri::command]
    pub async fn generate_key_pair_cmd(input: GenKeyInput) -> Result<(Vec<u8>, String), String> {
        let key_pair = if let Some(seed) = input.seed {
            let mut seed_bytes = [0u8; 32];
            let seed_hash = blake3::hash(seed.as_bytes());
            seed_bytes.copy_from_slice(&seed_hash.as_bytes()[..32]);
            KeyPair::from_seed(&seed_bytes)
        } else {
            KeyPair::generate()
        };

        let pkcs8 = key_pair.to_pkcs8();
        let pubkey = key_pair.verifying_key_bytes();
        Ok((pkcs8, pubkey))
    }

    #[derive(serde::Deserialize)]
    pub struct SignInput {
        pub private_key_pkcs8: Vec<u8>,
        pub data: String,
    }

    #[tauri::command]
    pub async fn sign_data_cmd(input: SignInput) -> Result<String, String> {
        let key_pair = KeyPair::from_pkcs8(&input.private_key_pkcs8)
            .map_err(|e| e.to_string())?;
        let signature = key_pair.sign(input.data.as_bytes());
        Ok(hex::encode(signature.to_bytes()))
    }

    #[derive(serde::Deserialize)]
    pub struct VerifyInput {
        pub public_key_hex: String,
        pub data: String,
        pub signature_hex: String,
    }

    #[tauri::command]
    pub async fn verify_signature_cmd(input: VerifyInput) -> Result<bool, String> {
        let pubkey_bytes = hex::decode(&input.public_key_hex)
            .map_err(|e| e.to_string())?;
        let signature_bytes = hex::decode(&input.signature_hex)
            .map_err(|e| e.to_string())?;
        let data = input.data.as_bytes();

        crate::cert_gen::verify_signature(&pubkey_bytes, data, &signature_bytes)
            .map_err(|e| e.to_string())
    }
}