use std::process::Command;
use anyhow::{Result, Context};

#[derive(Debug, Clone)]
pub enum Encoder {
    Nvidia,
    Amd,
    Intel,
    Software,
}

fn detect_gpu_vendor() -> Option<String> {
    let output = Command::new("wmic")
        .args(["path", "win32_VideoController", "get", "Name"])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();

    if stdout.contains("nvidia") {
        Some("nvidia".to_string())
    } else if stdout.contains("amd") || stdout.contains("radeon") {
        Some("amd".to_string())
    } else if stdout.contains("intel") {
        Some("intel".to_string())
    } else {
        None
    }
}

pub fn detect_encoder() -> Result<Encoder> {
    let output = Command::new("ffmpeg")
        .arg("-hide_banner")
        .arg("-encoders")
        .output()
        .context("Failed to run ffmpeg to detect encoders")?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    match detect_gpu_vendor().as_deref() {
        Some("nvidia") if stdout.contains("h264_nvenc") => Ok(Encoder::Nvidia),
        Some("amd") if stdout.contains("h264_amf") => Ok(Encoder::Amd),
        Some("intel") if stdout.contains("h264_qsv") => Ok(Encoder::Intel),
        _ => Ok(Encoder::Software),
    }
}