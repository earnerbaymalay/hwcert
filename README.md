# HWCert

Offline hardware diagnostics for IT techs: inventory a machine, stress it, benchmark a local LLM on it, check firmware, then issue an Ed25519-signed certificate (JSON + QR + PDF) that anyone can verify at [earnerbaymalay.github.io/hwcert-verifier](https://earnerbaymalay.github.io/hwcert-verifier/).

## Crates

| Crate | What it does | Status |
|-------|--------------|--------|
| `hw-inventory` | CPU, memory, storage, system, battery via `sysinfo`; NVIDIA GPUs via NVML (`--features nvml`) | USB/PCI enumeration and SMART data are stubs |
| `hw-stress` | CPU, memory and disk stress with a thermal log | GPU stress is a placeholder (`--features gpu`); temperature sensors not read yet |
| `llm-bench` | LLM benchmark: tok/s, TTFT, TPOT, latency percentiles | **Ollama** backend works; candle / llama.cpp / ONNX are stubs |
| `fw-check` | SMBIOS info + firmware update database lookup | Update DB not published yet; use `--check-db false` offline |
| `cert-gen` | Ed25519 signing/verification, QR PNG, one-page PDF | Done |
| `diagbox` | CLI over all of the above | Done |
| `tauri-app` | Desktop UI over all of the above | Done (basic) |

## Quick start

```bash
cargo run -p diagbox -- inventory -o inv.json
cargo run -p diagbox -- stress --help
cargo run -p diagbox -- bench --runs 5                     # Ollama, default model from palexus-brain
cargo run -p diagbox -- bench -m qwen3.5:2b --runs 5 -o bench.json
cargo run -p diagbox -- firmware --check-db false
cargo run -p diagbox -- gen-key -o key
cargo run -p diagbox -- cert -t hardware -i inv.json -k key -o cert.json
cargo run -p diagbox -- verify -i cert.json -p key.pub
cargo run -p hwcert-tauri                                  # desktop app
```

`bench` needs a running Ollama (`ollama serve`). The endpoint and default model come from the shared **palexus-brain** config (`palexus-brain config show`), e.g. `PALEXUS_BRAIN_INFERENCE__DEFAULT_MODEL=qwen3.5:2b`.

## Build

Rust stable. This repo builds inside the `ai-cli-work` workspace: it depends on `../shared/ollama-client` and `../shared/palexus-brain-schema`.

```bash
cargo build --workspace
cargo test --workspace
```

Optional features: `nvml` (hw-inventory), `gpu` (hw-stress), `llama` / `onnx` (llm-bench, diagbox, tauri-app; need extra native toolchains).

## License

MIT OR Apache-2.0
