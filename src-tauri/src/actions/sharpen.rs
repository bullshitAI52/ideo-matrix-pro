use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct SharpenAction;

impl VideoAction for SharpenAction {
    fn id(&self) -> &'static str {
        "sharpen"
    }

    fn execute(&self, src: &Path, out_dir: &Path, config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "sharp")?;
        
        let strength = config.params.get("sharpen_strength").and_then(|v| v.as_f64()).unwrap_or(1.0);
        let vf = format!("unsharp=5:5:{}:5:5:0.0", strength);
        
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
