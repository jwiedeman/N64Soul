use std::env;
use std::path::PathBuf;
use std::process::Command;

fn env_flag(value: &str) -> bool {
    matches!(value.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on")
}

fn run_command(mut cmd: Command, description: &str) {
    let status = cmd
        .status()
        .unwrap_or_else(|err| panic!("failed to launch {}: {}", description, err));
    if !status.success() {
        panic!("{} exited with {}", description, status);
    }
}

fn main() {
    println!("cargo:rerun-if-env-changed=N64_SOUL_SKIP_EXPORT");
    println!("cargo:rerun-if-env-changed=N64_SOUL_MODEL_ID");
    println!("cargo:rerun-if-env-changed=N64_SOUL_DTYPE");
    println!("cargo:rerun-if-env-changed=N64_SOUL_KEEP_LAYERS");
    println!("cargo:rerun-if-env-changed=N64_SOUL_TUNE_CONFIG");
    println!("cargo:rerun-if-env-changed=N64_SOUL_EXPORT_SCRIPT");
    println!("cargo:rerun-if-env-changed=PYTHON");

    if env::var_os("CARGO_FEATURE_EMBED_ASSETS").is_none() {
        return;
    }

    if env::var("N64_SOUL_SKIP_EXPORT")
        .ok()
        .map(|v| env_flag(&v))
        .unwrap_or(false)
    {
        println!("cargo:warning=skipping model export because N64_SOUL_SKIP_EXPORT is set");
        return;
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR missing"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(PathBuf::from)
        .unwrap_or_else(|| manifest_dir.clone());

    let script_path = env::var("N64_SOUL_EXPORT_SCRIPT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| repo_root.join("tools").join("export_gpt2_n64.py"));

    if !script_path.exists() {
        panic!("export script not found at {}", script_path.display());
    }

    let python = env::var("PYTHON").unwrap_or_else(|_| "python3".to_string());
    let assets_dir = manifest_dir.join("assets");
    std::fs::create_dir_all(&assets_dir).expect("failed to create assets directory");

    let mut export_cmd = Command::new(&python);
    export_cmd.arg(&script_path);
    export_cmd.arg("--out-dir").arg(&assets_dir);

    let model_id = env::var("N64_SOUL_MODEL_ID").unwrap_or_else(|_| "gpt2".to_string());
    export_cmd.arg("--model").arg(model_id);

    let dtype = env::var("N64_SOUL_DTYPE").unwrap_or_else(|_| "fp16".to_string());
    export_cmd.arg("--dtype").arg(dtype);

    if let Ok(keep_layers) = env::var("N64_SOUL_KEEP_LAYERS") {
        if !keep_layers.trim().is_empty() {
            export_cmd.arg("--keep-layers").arg(keep_layers);
        }
    }

    if let Ok(tune_cfg) = env::var("N64_SOUL_TUNE_CONFIG") {
        if !tune_cfg.trim().is_empty() {
            export_cmd.arg("--tune-config").arg(tune_cfg);
        }
    }

    println!(
        "cargo:warning=exporting model weights via {}",
        script_path.display()
    );
    run_command(export_cmd, "export_gpt2_n64.py");

    let weights_bin = assets_dir.join("weights.bin");
    let weights_manifest = assets_dir.join("weights.manifest.bin");

    println!(
        "cargo:warning=validating manifest {}",
        weights_manifest.display()
    );

    let validator = repo_root.join("tools").join("validate_weights.py");
    if validator.exists() {
        let mut validate_cmd = Command::new(&python);
        validate_cmd
            .arg(&validator)
            .arg("--bin")
            .arg(&weights_bin)
            .arg("--man")
            .arg(&weights_manifest)
            .arg("--crc");
        run_command(validate_cmd, "validate_weights.py");
    } else {
        println!(
            "cargo:warning=validator script missing at {}; skipping layout check",
            validator.display()
        );
    }

    println!("cargo:rerun-if-changed={}", script_path.display());
    println!("cargo:rerun-if-changed={}", weights_bin.display());
    println!("cargo:rerun-if-changed={}", weights_manifest.display());
}
