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
            let escaped_path = FFUtils::escape_path(path);
            
            // Improved filter graph for robustness:
            // 1. movie='...':loop=0 -> Loads mask video and loops it indefinitely so it covers full duration
            // 2. scale2ref -> Scales the mask video (first input) to match the main video dimensions (second input)
            // 3. blend -> Applies the blend effect
            // 4. shortest=1 -> Ensures output stops when the main video ends (important since mask is now infinite)
            let vf = format!("movie='{}':loop=0[mask];[mask][in]scale2ref[mask_scaled][in_main];[in_main][mask_scaled]blend=all_mode=multiply:shortest=1", escaped_path);
            
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
