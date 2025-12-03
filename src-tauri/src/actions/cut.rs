use std::path::Path;
use anyhow::{Result, anyhow};
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct CutAction;

impl VideoAction for CutAction {
    fn id(&self) -> &'static str {
        "cut_head_tail"
    }

    fn execute(&self, src: &Path, out_dir: &Path, _config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "cut")?;
        
        let duration = FFUtils::get_duration(src)?;
        if duration < 3.0 {
            return Err(anyhow!("Video too short for cutting (min 3s)"));
        }

        let new_duration = duration - 2.0;
        
        FFUtils::run(&[
            "-y",
            "-ss", "1",
            "-t", &new_duration.to_string(),
            "-i", src.to_str().unwrap(),
            "-c", "copy",
            "-loglevel", "error",
            dst.to_str().unwrap()
        ])
    }
}
