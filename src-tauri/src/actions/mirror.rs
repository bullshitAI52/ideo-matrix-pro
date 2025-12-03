use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct MirrorAction;

impl VideoAction for MirrorAction {
    fn id(&self) -> &'static str {
        "mirror"
    }

    fn execute(&self, src: &Path, out_dir: &Path, _config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "flip")?;
        
        FFUtils::run(&[
            "-y",
            "-i", src.to_str().unwrap(),
            "-vf", "hflip",
            "-c:a", "copy",
            "-loglevel", "error",
            dst.to_str().unwrap()
        ])
    }
}
