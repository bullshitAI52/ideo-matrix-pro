use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct GrainAction;

impl VideoAction for GrainAction {
    fn id(&self) -> &'static str {
        "grain"
    }

    fn execute(&self, src: &Path, out_dir: &Path, config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "grain")?;
        
        let strength = config.params.get("grain_strength").and_then(|v| v.as_f64()).unwrap_or(0.1);
        // Scale 0.0-0.5 to 0-50 for noise filter
        let noise_val = (strength * 100.0) as i32;
        let vf = format!("noise=alls={}:allf=t+u", noise_val);
        
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
