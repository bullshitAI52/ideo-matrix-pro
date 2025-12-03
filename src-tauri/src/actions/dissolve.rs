use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct DissolveAction;

impl VideoAction for DissolveAction {
    fn id(&self) -> &'static str {
        "dissolve"
    }

    fn execute(&self, src: &Path, out_dir: &Path, _config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "ai_dis")?;
        
        let duration = FFUtils::get_duration(src)?;
        let vf = format!("fade=t=in:st=0:d=1,fade=t=out:st={}:d=1", duration - 1.0);
        
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
