use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct VignetteAction;

impl VideoAction for VignetteAction {
    fn id(&self) -> &'static str {
        "vignette"
    }

    fn execute(&self, src: &Path, out_dir: &Path, config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "vig")?;
        
        let strength = config.params.get("vignette_strength").and_then(|v| v.as_f64()).unwrap_or(0.5);
        // strength 0.0-1.0 maps to angle 0 to PI/2
        let angle = strength * std::f64::consts::PI / 2.0;
        let vf = format!("vignette={:.3}", angle);
        
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
