use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct ScanAction;

impl VideoAction for ScanAction {
    fn id(&self) -> &'static str {
        "scan"
    }

    fn execute(&self, src: &Path, out_dir: &Path, _config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "scan")?;
        
        let vf = "eq=brightness='0.08*sin(2*PI*t/3)'";
        
        FFUtils::run(&[
            "-y",
            "-i", src.to_str().unwrap(),
            "-vf", vf,
            "-c:a", "copy",
            "-loglevel", "error",
            dst.to_str().unwrap()
        ])
    }
}
