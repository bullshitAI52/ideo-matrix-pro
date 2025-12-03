use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct PullAction;

impl VideoAction for PullAction {
    fn id(&self) -> &'static str {
        "pull"
    }

    fn execute(&self, src: &Path, out_dir: &Path, _config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "pull")?;
        
        // select='not(mod(n,30))',setpts=N/FRAME_RATE/TB
        let vf = "select='not(mod(n,30))',setpts=N/FRAME_RATE/TB";
        
        FFUtils::run(&[
            "-y",
            "-i", src.to_str().unwrap(),
            "-vf", vf,
            "-an",
            "-loglevel", "error",
            dst.to_str().unwrap()
        ])
    }
}
