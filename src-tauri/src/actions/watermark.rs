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
            // Image Watermark (Top-Right corner, 10px padding, 30% opacity)
            // overlay=W-w-10:10:format=auto, colorchannelmixer=aa=0.3
            // Note: complex filter needed for opacity
            let vf = format!("movie='{}'[wm];[in][wm]overlay=W-w-10:10", path);
            
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
