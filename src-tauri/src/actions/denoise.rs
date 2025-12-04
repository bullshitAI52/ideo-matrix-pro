use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct DenoiseAction;

impl VideoAction for DenoiseAction {
    fn id(&self) -> &'static str {
        "denoise"
    }

    fn execute(&self, src: &Path, out_dir: &Path, config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "denoise")?;
        
        let strength = config.params.get("denoise_strength").and_then(|v| v.as_f64()).unwrap_or(5.0);
        // hqdn3d=luma_spatial:chroma_spatial:luma_tmp:chroma_tmp
        // We scale all parameters based on strength
        let vf = format!("hqdn3d={0}:{0}:{1}:{1}", strength * 0.3, strength);
        
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
