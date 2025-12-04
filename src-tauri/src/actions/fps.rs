use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct FpsAction;

impl VideoAction for FpsAction {
    fn id(&self) -> &'static str {
        "fps_60"
    }

    fn execute(&self, src: &Path, out_dir: &Path, config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "fps")?;
        
        let fps = config.params.get("target_fps").and_then(|v| v.as_u64()).unwrap_or(60).to_string();
        
        FFUtils::run(&[
            "-y",
            "-i", src.to_str().unwrap(),
            "-r", &fps,
            "-loglevel", "error",
            dst.to_str().unwrap()
        ])
    }
}
