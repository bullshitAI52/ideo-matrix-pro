use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct MaskVideoAction;

impl VideoAction for MaskVideoAction {
    fn id(&self) -> &'static str {
        "mask_video"
    }

    fn execute(&self, src: &Path, out_dir: &Path, config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "mask_video")?;
        
        if let Some(path) = &config.mask_video_path {
            // Mask video overlay with blend mode
            let escaped_path = FFUtils::escape_path(path);
            
            // Use blend filter for video mask effect
            // This creates a more dynamic mask effect compared to static image
            let vf = format!("movie='{}'[mask];[in][mask]blend=all_mode=multiply", escaped_path);
            
            FFUtils::run(&[
                "-i", src.to_str().unwrap(),
                "-vf", &vf,
                "-c:a", "copy",
                dst.to_str().unwrap()
            ])
        } else {
            // Fallback: copy
            FFUtils::run(&[
                "-i", src.to_str().unwrap(),
                "-c", "copy",
                dst.to_str().unwrap()
            ])
        }
    }
}
