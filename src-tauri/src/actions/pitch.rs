use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct PitchAction;

impl VideoAction for PitchAction {
    fn id(&self) -> &'static str {
        "pitch"
    }

    fn execute(&self, src: &Path, out_dir: &Path, _config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "pitch")?;
        
        FFUtils::run(&[
            "-y",
            "-i", src.to_str().unwrap(),
            "-af", "asetrate=44100*1.05,aresample=44100",
            "-c:v", "copy",
            "-loglevel", "error",
            dst.to_str().unwrap()
        ])
    }
}
