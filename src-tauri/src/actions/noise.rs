use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct AudioNoiseAction;

impl VideoAction for AudioNoiseAction {
    fn id(&self) -> &'static str {
        "audio_noise"
    }

    fn execute(&self, src: &Path, out_dir: &Path, _config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "anoise")?;
        
        // aevalsrc=-2+random(0):d=50[n];[n]volume=0.03[vn];[0:a][vn]amix=inputs=2:duration=first
        let filter_complex = "aevalsrc=-2+random(0):d=50[n];[n]volume=0.03[vn];[0:a][vn]amix=inputs=2:duration=first";
        
        FFUtils::run(&[
            "-y",
            "-i", src.to_str().unwrap(),
            "-filter_complex", filter_complex,
            "-c:v", "copy",
            "-loglevel", "error",
            dst.to_str().unwrap()
        ])
    }
}
