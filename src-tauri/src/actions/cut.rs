use std::path::Path;
use anyhow::{Result, anyhow};
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct CutAction;

impl VideoAction for CutAction {
    fn id(&self) -> &'static str {
        "cut_head_tail"
    }

    fn execute(&self, src: &Path, out_dir: &Path, config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "cut")?;
        
        let cut_secs = config.params.get("cut_seconds").and_then(|v| v.as_f64()).unwrap_or(1.0);
        let duration = FFUtils::get_duration(src)?;
        
        if duration < cut_secs * 2.0 + 1.0 {
            return Err(anyhow!("Video too short for cutting"));
        }

        let new_duration = duration - cut_secs * 2.0;
        
        FFUtils::run(&[
            "-y",
            "-ss", &cut_secs.to_string(),
            "-t", &new_duration.to_string(),
            "-i", src.to_str().unwrap(),
            "-c", "copy",
            "-loglevel", "error",
            dst.to_str().unwrap()
        ])
    }
}
