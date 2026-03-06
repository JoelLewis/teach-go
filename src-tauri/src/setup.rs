use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tracing::info;

const KATAGO_VERSION: &str = "v1.15.3";

#[cfg(target_os = "linux")]
const BINARY_ASSET: &str = "katago-v1.15.3-cuda12-linux-x64.zip";
#[cfg(target_os = "macos")]
const BINARY_ASSET: &str = "katago-v1.15.3-eigen-macos-x64.zip";
#[cfg(target_os = "windows")]
const BINARY_ASSET: &str = "katago-v1.15.3-opencl-windows-x64.zip";

const MODEL_URL: &str = "https://media.katagotraining.org/uploaded/networks/models/kata1/kata1-b18c384nbt-s9996604416-d4316597426.bin.gz";
const MODEL_FILENAME: &str = "kata1-b18c384nbt-s9996604416-d4316597426.bin.gz";

const ANALYSIS_CFG: &[u8] = include_bytes!("../binaries/analysis.cfg");

#[cfg(target_os = "windows")]
const BINARY_NAME: &str = "katago.exe";
#[cfg(not(target_os = "windows"))]
const BINARY_NAME: &str = "katago";

#[derive(Clone, Serialize, Deserialize)]
pub struct SetupProgress {
    pub phase: String,
    pub downloaded: u64,
    pub total: u64,
}

/// Returns `"ready"`, `"partial"`, or `"not_installed"`.
pub fn setup_status(katago_dir: &Path) -> &'static str {
    let has_binary = katago_dir.join(BINARY_NAME).exists();
    let has_model = katago_dir.join(MODEL_FILENAME).exists();
    match (has_binary, has_model) {
        (true, true) => "ready",
        (true, false) | (false, true) => "partial",
        (false, false) => "not_installed",
    }
}

/// Download KataGo binary + model, write analysis.cfg. Calls `on_progress` throughout.
pub fn ensure_katago(
    katago_dir: &Path,
    on_progress: impl Fn(SetupProgress),
) -> Result<(), String> {
    fs::create_dir_all(katago_dir).map_err(|e| format!("create dir: {e}"))?;

    let binary_path = katago_dir.join(BINARY_NAME);
    if !binary_path.exists() {
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

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&binary_path, fs::Permissions::from_mode(0o755))
                .map_err(|e| format!("chmod: {e}"))?;
        }

        info!("KataGo binary extracted to {}", binary_path.display());
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

fn download_file(
    url: &str,
    dest: &Path,
    on_progress: impl Fn(u64, u64),
) -> Result<(), String> {
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
}

fn extract_katago_binary(zip_path: &Path, dest: &Path) -> Result<(), String> {
    let file = fs::File::open(zip_path).map_err(|e| format!("open zip: {e}"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("read zip: {e}"))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("zip entry: {e}"))?;

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
