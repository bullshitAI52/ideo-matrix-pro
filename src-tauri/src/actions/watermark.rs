use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

// Note: Watermark requires external image file - placeholder implementation
pub struct WatermarkAction;

impl VideoAction for WatermarkAction {
    fn id(&self) -> &'static str {
        "watermark"
    }

    fn execute(&self, src: &Path, out_dir: &Path, config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "watermark")?;
        
        if let Some(path) = &config.watermark_path {
            // Get parameters
            let position = config.params.get("watermark_position").and_then(|v| v.as_str()).unwrap_or("top_right");
            let opacity = config.params.get("watermark_opacity").and_then(|v| v.as_f64()).unwrap_or(0.5);
            
            // Calculate overlay coordinates
            let coord = match position {
                "top_left" => "10:10",
                "top_right" => "W-w-10:10",
                "bottom_left" => "10:H-h-10",
                "bottom_right" => "W-w-10:H-h-10",
                "center" => "(W-w)/2:(H-h)/2",
                _ => "W-w-10:10"
            };
            
            // Apply opacity and overlay
            // [wm]format=rgba,colorchannelmixer=aa={opacity}[wm_t];[in][wm_t]overlay={coord}
            let vf = format!("movie='{}',format=rgba,colorchannelmixer=aa={}[wm];[in][wm]overlay={}", path, opacity, coord);
            
            FFUtils::run(&[
                "-y",
                "-i", src.to_str().unwrap(),
                "-vf", &vf,
                "-c:a", "copy",
                "-loglevel", "error",
                dst.to_str().unwrap()
            ])
        } else {
            // Fallback: Text Watermark
            let vf = "drawtext=text='Processed':fontsize=24:fontcolor=white@0.5:x=10:y=10";
            
            FFUtils::run(&[
                "-y",
                "-i", src.to_str().unwrap(),
                "-vf", vf,
                "-c:a", "copy",
                "-loglevel", "error",
                dst.to_str().unwrap()
            ])
        }
    }
}
