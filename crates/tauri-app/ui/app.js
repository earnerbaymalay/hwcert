"use strict";

const { invoke } = window.__TAURI__.core;
const $ = (id) => document.getElementById(id);

const state = { lastResult: null, lastType: null, key: null, cert: null };

function show(value) {
  $("output").textContent =
    typeof value === "string" ? value : JSON.stringify(value, null, 2);
}

// Runs a backend command with the button disabled; reports errors in the output pane.
async function run(button, label, fn) {
  button.disabled = true;
  $("status").textContent = `${label}…`;
  try {
    show(await fn());
    $("status").textContent = `${label}: done`;
  } catch (err) {
    show(`Error: ${err}`);
    $("status").textContent = `${label}: failed`;
  } finally {
    button.disabled = false;
  }
}

function remember(type) {
  return (result) => {
    state.lastResult = result;
    state.lastType = type;
    return result;
  };
}

$("btn-inventory").addEventListener("click", (e) =>
  run(e.target, "Inventory", () => invoke("gather_inventory_cmd").then(remember("hardware")))
);

$("btn-firmware").addEventListener("click", (e) =>
  run(e.target, "Firmware", () => invoke("gather_firmware_report_cmd").then(remember("firmware")))
);

$("btn-stress").addEventListener("click", (e) => {
  const mem = parseFloat($("stress-mem").value);
  const config = {
    cpu_threads: null,
    duration_secs: Math.max(1, parseInt($("stress-duration").value, 10) || 30),
    memory_gb: mem > 0 ? mem : null,
    disk_path: null,
    disk_gb: null,
    cooldown_secs: 5,
    thermal_limit_celsius: 95,
  };
  run(e.target, "Stress test", () => invoke("run_stress_test_cmd", { config }).then(remember("stress_test")));
});

$("btn-bench").addEventListener("click", (e) => {
  const config = {
    model_path: $("bench-model").value.trim(),
    backend: $("bench-backend").value,
    prompt: "Explain what a hardware stress test measures.",
    max_tokens: 128,
    runs: 3,
    temperature: 0.7,
    gpu_layers: null,
    threads: null,
  };
  if (!config.model_path) return show("Enter a model path first.");
  run(e.target, "Benchmark", () => invoke("run_benchmark_cmd", { config }));
});

$("btn-keygen").addEventListener("click", (e) =>
  run(e.target, "Key pair", async () => {
    const [secret, publicKeyHex] = await invoke("generate_key_pair_cmd", { input: { seed: null } });
    state.key = { secret, publicKeyHex };
    return { public_key: publicKeyHex, note: "Secret key held in memory only for this session." };
  })
);

$("btn-cert").addEventListener("click", (e) => {
  if (!state.key) return show("Generate a key pair first.");
  if (!state.lastResult) return show("Run inventory, firmware, or a stress test first.");
  run(e.target, "Certificate", async () => {
    const cert = await invoke("generate_certificate_cmd", {
      input: { cert_type: state.lastType, payload: state.lastResult, private_key_pkcs8: state.key.secret },
    });
    state.cert = cert;
    // Leave the PDF bytes out of the on-screen view.
    return { ...cert, pdf_data: cert.pdf_data ? `<${cert.pdf_data.length} bytes>` : null };
  });
});

$("btn-verify").addEventListener("click", (e) => {
  if (!state.cert) return show("Generate a certificate first.");
  run(e.target, "Verify", async () => {
    const ok = await invoke("verify_certificate_cmd", {
      input: { certificate: state.cert, public_key_hex: state.cert.public_key },
    });
    return ok ? "Signature valid." : "Signature INVALID.";
  });
});
