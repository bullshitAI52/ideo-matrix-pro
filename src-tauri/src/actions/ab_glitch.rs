use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct AbGlitchAction;

impl VideoAction for AbGlitchAction {
    fn id(&self) -> &'static str {
        "ab_glitch"
    }

    fn execute(&self, src: &Path, out_dir: &Path, _config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "ab_glitch")?;
        
        // Glitch effect using noise and color shift
        let vf = "noise=alls=20:allf=t,hue=s=0.8";
        
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
