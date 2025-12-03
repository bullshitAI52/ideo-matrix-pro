use std::path::Path;
use std::process::Command;
use anyhow::{Result, anyhow};

pub struct FFUtils;

impl FFUtils {
    /// Run an FFmpeg command
    pub fn run(args: &[&str]) -> Result<()> {
        // In a real app, we might want to capture output or progress
        // For now, we just run it and check status
        
        // On Windows, we might need to ensure ffmpeg.exe is in PATH or bundled
        // For this refactor, we assume it's in PATH as per original requirements
        
        let status = Command::new("ffmpeg")
            .args(args)
            .status()
            .map_err(|e| anyhow!("Failed to execute ffmpeg: {}", e))?;

        if status.success() {
            Ok(())
        } else {
            Err(anyhow!("FFmpeg command failed with status: {}", status))
        }
    }

    /// Helper to generate output path with suffix
    pub fn get_dst(src: &Path, out_dir: &Path, suffix: &str) -> Result<std::path::PathBuf> {
        let file_stem = src.file_stem()
            .ok_or_else(|| anyhow!("Invalid source filename"))?
            .to_str()
            .ok_or_else(|| anyhow!("Invalid source filename encoding"))?;
            
        let ext = src.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("mp4");

        Ok(out_dir.join(format!("{}_{}.{}", file_stem, suffix, ext)))
    }

    /// Get video duration using ffprobe
    pub fn get_duration(src: &Path) -> Result<f64> {
        let output = Command::new("ffprobe")
            .args(&[
                "-v", "error",
                "-show_entries", "format=duration",
                "-of", "default=noprint_wrappers=1:nokey=1",
                src.to_str().unwrap()
            ])
            .output()
            .map_err(|e| anyhow!("Failed to execute ffprobe: {}", e))?;

        if !output.status.success() {
            return Err(anyhow!("ffprobe failed"));
        }

        let output_str = String::from_utf8(output.stdout)?;
        output_str.trim().parse::<f64>().map_err(|e| anyhow!("Failed to parse duration: {}", e))
    }
}
