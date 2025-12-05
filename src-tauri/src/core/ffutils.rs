use std::path::Path;
use tauri::api::process::Command;
use anyhow::{Result, anyhow};

pub struct FFUtils;

impl FFUtils {
    /// Run an FFmpeg command
    pub fn run(args: &[&str]) -> Result<()> {
        // Add -y to force overwrite
        let mut final_args = vec!["-y"];
        final_args.extend_from_slice(args);
        
        let output = Command::new_sidecar("ffmpeg")?
            .args(&final_args)
            .output()
            .map_err(|e| anyhow!("Failed to execute ffmpeg: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(anyhow!("FFmpeg failed: {}", output.stderr))
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
        let output = Command::new_sidecar("ffprobe")?
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

        let output_str = output.stdout;
        output_str.trim().parse::<f64>().map_err(|e| anyhow!("Failed to parse duration: {}", e))
    }

    /// Escape path for use in FFmpeg filter graph
    pub fn escape_path(path: &str) -> String {
        path.replace('\\', "/")
            .replace(':', "\\:")
            .replace('\'', "\\\'")
    }
}
