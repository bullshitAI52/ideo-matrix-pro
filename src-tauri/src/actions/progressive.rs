use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct ProgressiveAction;

impl VideoAction for ProgressiveAction {
    fn id(&self) -> &'static str {
        "progressive"
    }

    fn execute(&self, src: &Path, out_dir: &Path, _config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "prog")?;
        
        let duration = FFUtils::get_duration(src)?;
        let vf = format!("fade=t=in:st=0:d=0.5,fade=t=out:st={}:d=0.5,eq=contrast='1+0.1*sin(2*PI*t/2)'", duration - 0.5);
        
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
