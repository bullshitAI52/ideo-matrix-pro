use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

// Note: Watermark requires external image file - placeholder implementation
pub struct WatermarkAction;

impl VideoAction for WatermarkAction {
    fn id(&self) -> &'static str {
        "watermark"
    }

    fn execute(&self, src: &Path, out_dir: &Path, _config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "watermark")?;
        
        // Placeholder: Add text watermark instead of image
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
