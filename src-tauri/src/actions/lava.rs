use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct LavaAction;

impl VideoAction for LavaAction {
    fn id(&self) -> &'static str {
        "lava"
    }

    fn execute(&self, src: &Path, out_dir: &Path, _config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "ab_lava")?;
        
        let vf = "eq=contrast='1+0.3*sin(t)':saturation='1.5'";
        
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
