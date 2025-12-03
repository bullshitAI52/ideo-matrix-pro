use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct DenoiseAction;

impl VideoAction for DenoiseAction {
    fn id(&self) -> &'static str {
        "denoise"
    }

    fn execute(&self, src: &Path, out_dir: &Path, _config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "denoise")?;
        
        FFUtils::run(&[
            "-y",
            "-i", src.to_str().unwrap(),
            "-vf", "hqdn3d=1.5:1.5:6:6",
            "-c:a", "copy",
            "-loglevel", "error",
            dst.to_str().unwrap()
        ])
    }
}
