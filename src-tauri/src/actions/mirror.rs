use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct MirrorAction;

impl VideoAction for MirrorAction {
    fn id(&self) -> &'static str {
        "mirror"
    }

    fn execute(&self, src: &Path, out_dir: &Path, config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "flip")?;
        
        let direction = config.params.get("mirror_direction").and_then(|v| v.as_str()).unwrap_or("horizontal");
        let vf = match direction {
            "vertical" => "vflip",
            "both" => "hflip,vflip",
            _ => "hflip"
        };
        
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
