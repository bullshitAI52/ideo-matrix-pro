use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct AbBlendAction;

impl VideoAction for AbBlendAction {
    fn id(&self) -> &'static str {
        "ab_blend"
    }

    fn execute(&self, src: &Path, out_dir: &Path, _config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "ab_blend")?;
        
        // Simple blend effect using overlay
        let filter_complex = "[0:v]split=2[a][b];[a][b]blend=all_mode=overlay:all_opacity=0.5";
        
        FFUtils::run(&[
            "-y",
            "-i", src.to_str().unwrap(),
            "-filter_complex", filter_complex,
            "-c:a", "copy",
            "-loglevel", "error",
            dst.to_str().unwrap()
        ])
    }
}
