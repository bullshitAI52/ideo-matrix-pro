use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct MuteAction;

impl VideoAction for MuteAction {
    fn id(&self) -> &'static str {
        "mute"
    }

    fn execute(&self, src: &Path, out_dir: &Path, _config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "mute")?;
        
        FFUtils::run(&[
            "-y",
            "-i", src.to_str().unwrap(),
            "-c:v", "copy",
            "-an",
            "-loglevel", "error",
            dst.to_str().unwrap()
        ])
    }
}
