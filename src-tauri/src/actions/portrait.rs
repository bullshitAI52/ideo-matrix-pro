use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct PortraitAction;

impl VideoAction for PortraitAction {
    fn id(&self) -> &'static str {
        "portrait"
    }

    fn execute(&self, src: &Path, out_dir: &Path, config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "portrait")?;
        
        let strength = config.params.get("portrait_strength").and_then(|v| v.as_f64()).unwrap_or(2.0);
        let vf = format!("unsharp=7:7:{}:7:7:0.0,eq=contrast=1.1:brightness=0.02", strength);
        
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
