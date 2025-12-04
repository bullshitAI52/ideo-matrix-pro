use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct BitrateAction;

impl VideoAction for BitrateAction {
    fn id(&self) -> &'static str {
        "bitrate_hq"
    }

    fn execute(&self, src: &Path, out_dir: &Path, config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "hq")?;
        
        let bitrate = config.params.get("target_bitrate").and_then(|v| v.as_str()).unwrap_or("15M");
        // Simple bufsize calculation (2x bitrate) - this is a rough approximation
        let bufsize = format!("{}M", bitrate.trim_end_matches('M').parse::<f32>().unwrap_or(15.0) * 2.0);
        
        FFUtils::run(&[
            "-y",
            "-i", src.to_str().unwrap(),
            "-b:v", bitrate,
            "-minrate", bitrate,
            "-bufsize", &bufsize,
            "-loglevel", "error",
            dst.to_str().unwrap()
        ])
    }
}
