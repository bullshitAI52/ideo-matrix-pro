use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct BlurAction;

impl VideoAction for BlurAction {
    fn id(&self) -> &'static str {
        "blur"
    }

    fn execute(&self, src: &Path, out_dir: &Path, config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "blur")?;
        
        let sigma = config.params.get("blur_strength").and_then(|v| v.as_f64()).unwrap_or(0.5);
        let vf = format!("gblur=sigma={}", sigma);
        
        FFUtils::run(&[
            "-y",
            "-i", src.to_str().unwrap(),
            "-vf", &vf,
            "-c:a", "copy",
            "-loglevel", "error",
            dst.to_str().unwrap()
        ])
    }
}
