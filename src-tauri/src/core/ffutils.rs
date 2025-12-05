use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Result, anyhow};
use std::env;

pub struct FFUtils;

impl FFUtils {
    /// Get the path to bundled FFmpeg executable
    pub fn get_ffmpeg_path() -> PathBuf {
        // Try to find FFmpeg in the application directory
        if let Ok(exe_path) = env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                // Check for bundled FFmpeg
                let bundled_ffmpeg = exe_dir.join("ffmpeg.exe");
                if bundled_ffmpeg.exists() {
                    return bundled_ffmpeg;
                }
                
                // Also check in a "bin" subdirectory
                let bin_ffmpeg = exe_dir.join("bin").join("ffmpeg.exe");
                if bin_ffmpeg.exists() {
                    return bin_ffmpeg;
                }
            }
        }
        
        // Fallback to system FFmpeg
        PathBuf::from("ffmpeg")
    }
    
    /// Get the path to bundled FFprobe executable
    fn get_ffprobe_path() -> PathBuf {
        if let Ok(exe_path) = env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let bundled_ffprobe = exe_dir.join("ffprobe.exe");
                if bundled_ffprobe.exists() {
                    return bundled_ffprobe;
                }
                
                let bin_ffprobe = exe_dir.join("bin").join("ffprobe.exe");
                if bin_ffprobe.exists() {
                    return bin_ffprobe;
                }
            }
        }
        
        PathBuf::from("ffprobe")
    }

    /// Run an FFmpeg command
    pub fn run(args: &[&str]) -> Result<()> {
        let mut final_args = vec!["-y"];
        final_args.extend_from_slice(args);
        
        let ffmpeg_path = Self::get_ffmpeg_path();
        
        let output = Command::new(&ffmpeg_path)
            .args(&final_args)
            .output()
            .map_err(|e| anyhow!("Failed to execute ffmpeg at {:?}: {}", ffmpeg_path, e))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("FFmpeg failed: {}", stderr))
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
        let ffprobe_path = Self::get_ffprobe_path();
        
        let output = Command::new(&ffprobe_path)
            .args(&[
                "-v", "error",
                "-show_entries", "format=duration",
                "-of", "default=noprint_wrappers=1:nokey=1",
                src.to_str().unwrap()
            ])
            .output()
            .map_err(|e| anyhow!("Failed to execute ffprobe at {:?}: {}", ffprobe_path, e))?;

        if !output.status.success() {
            return Err(anyhow!("ffprobe failed"));
        }

        let output_str = String::from_utf8(output.stdout)?;
        output_str.trim().parse::<f64>().map_err(|e| anyhow!("Failed to parse duration: {}", e))
    }

    /// Escape path for use in FFmpeg filter graph
    pub fn escape_path(path: &str) -> String {
        path.replace('\\', "/")
            .replace(':', "\\:")
            .replace('\'', "\\\\'")
    }
}
