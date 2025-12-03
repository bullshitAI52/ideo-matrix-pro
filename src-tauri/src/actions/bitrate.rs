use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct BitrateAction;

impl VideoAction for BitrateAction {
    fn id(&self) -> &'static str {
        "bitrate_hq"
    }

    fn execute(&self, src: &Path, out_dir: &Path, _config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "hq")?;
        
        FFUtils::run(&[
            "-y",
            "-i", src.to_str().unwrap(),
            "-b:v", "15M",
            "-minrate", "15M",
            "-bufsize", "30M",
            "-loglevel", "error",
            dst.to_str().unwrap()
        ])
    }
}
