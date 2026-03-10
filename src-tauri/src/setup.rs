use std::fs;
#[cfg(target_os = "linux")]
use std::io;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::info;

#[cfg(target_os = "linux")]
const KATAGO_VERSION: &str = "v1.16.4";
#[cfg(target_os = "linux")]
const BINARY_ASSET: &str = "katago-v1.16.4-cuda12.1-cudnn8.9.7-linux-x64.zip";

/// Self-hosted Metal build for macOS (no official macOS release exists).
/// Built via .github/workflows/build-katago.yml and uploaded to GoSensei releases.
#[cfg(target_os = "macos")]
const MACOS_BINARY_URL: &str =
    "https://github.com/JoelLewis/teach-go/releases/download/katago-builds/katago";

const MODEL_URL: &str = "https://media.katagotraining.org/uploaded/networks/models/kata1/kata1-b18c384nbt-s9996604416-d4316597426.bin.gz";
const MODEL_FILENAME: &str = "kata1-b18c384nbt-s9996604416-d4316597426.bin.gz";

const ANALYSIS_CFG: &[u8] = include_bytes!("../binaries/analysis.cfg");

/// Expected SHA256 digests for integrity verification.
/// Set via environment variables at build time. When set, downloads are verified
/// after completion. Update these when upgrading KataGo or the model.
#[cfg(target_os = "macos")]
const KATAGO_BINARY_SHA256: Option<&str> = option_env!("KATAGO_BINARY_SHA256");
const MODEL_SHA256: Option<&str> = option_env!("KATAGO_MODEL_SHA256");

const BINARY_NAME: &str = "katago";

#[derive(Clone, Serialize, Deserialize)]
pub struct SetupProgress {
    pub phase: String,
    pub downloaded: u64,
    pub total: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KataGoStatus {
    Ready,
    Partial,
    NotInstalled,
}

pub fn setup_status(katago_dir: &Path) -> KataGoStatus {
    let has_binary = katago_dir.join(BINARY_NAME).exists();
    let has_model = katago_dir.join(MODEL_FILENAME).exists();
    match (has_binary, has_model) {
        (true, true) => KataGoStatus::Ready,
        (true, false) | (false, true) => KataGoStatus::Partial,
        (false, false) => KataGoStatus::NotInstalled,
    }
}

/// Download KataGo binary + model, write analysis.cfg. Calls `on_progress` throughout.
pub fn ensure_katago(katago_dir: &Path, on_progress: impl Fn(SetupProgress)) -> Result<(), String> {
    fs::create_dir_all(katago_dir).map_err(|e| format!("create dir: {e}"))?;

    let binary_path = katago_dir.join(BINARY_NAME);
    if !binary_path.exists() {
        #[cfg(target_os = "macos")]
        {
            // macOS: download pre-built Metal binary directly (no zip)
            info!("Downloading KataGo Metal binary from {MACOS_BINARY_URL}");
            download_file(MACOS_BINARY_URL, &binary_path, |downloaded, total| {
                on_progress(SetupProgress {
                    phase: "binary".into(),
                    downloaded,
                    total,
                });
            })?;
            verify_sha256(&binary_path, KATAGO_BINARY_SHA256)?;
        }

        #[cfg(target_os = "linux")]
        {
            // Linux: download zip from official KataGo releases
            let url = format!(
                "https://github.com/lightvector/KataGo/releases/download/{KATAGO_VERSION}/{BINARY_ASSET}"
            );
            info!("Downloading KataGo binary from {url}");

            let zip_path = katago_dir.join("katago-download.zip");
            download_file(&url, &zip_path, |downloaded, total| {
                on_progress(SetupProgress {
                    phase: "binary".into(),
                    downloaded,
                    total,
                });
            })?;

            extract_katago_binary(&zip_path, &binary_path)?;
            fs::remove_file(&zip_path).ok();
        }

        // Ad-hoc codesign so macOS doesn't block execution
        #[cfg(target_os = "macos")]
        {
            let output = std::process::Command::new("codesign")
                .args(["--sign", "-", "--force", &binary_path.to_string_lossy()])
                .output();
            match output {
                Ok(o) if o.status.success() => info!("Ad-hoc codesigned KataGo binary"),
                Ok(o) => tracing::warn!(
                    "codesign exited {}: {}",
                    o.status,
                    String::from_utf8_lossy(&o.stderr)
                ),
                Err(e) => tracing::warn!("codesign failed to run: {e}"),
            }
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&binary_path, fs::Permissions::from_mode(0o755))
                .map_err(|e| format!("chmod: {e}"))?;
        }

        info!("KataGo binary ready at {}", binary_path.display());
    }

    let model_path = katago_dir.join(MODEL_FILENAME);
    if !model_path.exists() {
        info!("Downloading KataGo neural net model");
        download_file(MODEL_URL, &model_path, |downloaded, total| {
            on_progress(SetupProgress {
                phase: "model".into(),
                downloaded,
                total,
            });
        })?;
        verify_sha256(&model_path, MODEL_SHA256)?;
        info!("Model saved to {}", model_path.display());
    }

    let cfg_path = katago_dir.join("analysis.cfg");
    if !cfg_path.exists() {
        fs::write(&cfg_path, ANALYSIS_CFG).map_err(|e| format!("write config: {e}"))?;
    }

    on_progress(SetupProgress {
        phase: "done".into(),
        downloaded: 0,
        total: 0,
    });

    Ok(())
}

fn download_file(url: &str, dest: &Path, on_progress: impl Fn(u64, u64)) -> Result<(), String> {
    let response = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .map_err(|e| format!("http client: {e}"))?
        .get(url)
        .send()
        .map_err(|e| format!("download request: {e}"))?
        .error_for_status()
        .map_err(|e| format!("download failed: {e}"))?;

    let total = response.content_length().unwrap_or(0);
    let mut reader = response;
    let mut file = fs::File::create(dest).map_err(|e| format!("create file: {e}"))?;
    let mut downloaded: u64 = 0;
    let mut buf = [0u8; 65_536];

    let result: Result<(), String> = (|| {
        loop {
            let n = reader.read(&mut buf).map_err(|e| format!("read: {e}"))?;
            if n == 0 {
                break;
            }
            file.write_all(&buf[..n])
                .map_err(|e| format!("write: {e}"))?;
            downloaded += n as u64;
            on_progress(downloaded, total);
        }
        file.flush().map_err(|e| format!("flush: {e}"))?;
        Ok(())
    })();

    if result.is_err() {
        // Remove partial file so next attempt starts fresh
        let _ = fs::remove_file(dest);
    }

    result
}

#[cfg(target_os = "linux")]
fn extract_katago_binary(zip_path: &Path, dest: &Path) -> Result<(), String> {
    let file = fs::File::open(zip_path).map_err(|e| format!("open zip: {e}"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("read zip: {e}"))?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| format!("zip entry: {e}"))?;

        let name = entry.name().to_string();
        let is_katago = name.ends_with(BINARY_NAME) && !name.contains("__MACOSX");

        if is_katago && !entry.is_dir() {
            let mut out = fs::File::create(dest).map_err(|e| format!("create binary: {e}"))?;
            io::copy(&mut entry, &mut out).map_err(|e| format!("extract: {e}"))?;
            return Ok(());
        }
    }

    Err("katago binary not found in zip archive".into())
}

/// Verify a file's SHA256 digest against an expected hex string.
/// Returns Ok(()) if the digest matches or if no expected digest is provided.
fn verify_sha256(path: &Path, expected: Option<&str>) -> Result<(), String> {
    let Some(expected_hex) = expected else {
        return Ok(());
    };

    let mut file = fs::File::open(path).map_err(|e| format!("open for verify: {e}"))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65_536];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| format!("read for verify: {e}"))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let digest = format!("{:x}", hasher.finalize());
    if digest != expected_hex {
        // Remove the corrupted file
        let _ = fs::remove_file(path);
        return Err(format!(
            "SHA256 mismatch for {}: expected {expected_hex}, got {digest}",
            path.display()
        ));
    }
    info!("SHA256 verified for {}", path.display());
    Ok(())
}

/// Path to the downloaded binary, if present.
pub fn binary_path(katago_dir: &Path) -> Option<PathBuf> {
    let p = katago_dir.join(BINARY_NAME);
    p.exists().then_some(p)
}

/// Path to the downloaded model, if present.
pub fn model_path(katago_dir: &Path) -> Option<PathBuf> {
    let p = katago_dir.join(MODEL_FILENAME);
    p.exists().then_some(p)
}

/// Path to the analysis config, if present.
pub fn config_path(katago_dir: &Path) -> Option<PathBuf> {
    let p = katago_dir.join("analysis.cfg");
    p.exists().then_some(p)
}
