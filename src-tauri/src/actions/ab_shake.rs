use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct AbShakeAction;

impl VideoAction for AbShakeAction {
    fn id(&self) -> &'static str {
        "ab_shake"
    }

    fn execute(&self, src: &Path, out_dir: &Path, _config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "ab_shake")?;
        
        // Shake effect using crop with sine wave movement
        let vf = "crop=iw:ih:5*sin(t*10):5*cos(t*10)";
        
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
