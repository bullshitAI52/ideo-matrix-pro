use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct AudioNoiseAction;

impl VideoAction for AudioNoiseAction {
    fn id(&self) -> &'static str {
        "audio_noise"
    }

    fn execute(&self, src: &Path, out_dir: &Path, config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "anoise")?;
        
        let strength = config.params.get("noise_strength").and_then(|v| v.as_f64()).unwrap_or(0.01);
        // aevalsrc=-2+random(0):d=50[n];[n]volume={strength}[vn];[0:a][vn]amix=inputs=2:duration=first
        let filter_complex = format!("aevalsrc=-2+random(0):d=50[n];[n]volume={}[vn];[0:a][vn]amix=inputs=2:duration=first", strength);
        
        FFUtils::run(&[
            "-y",
            "-i", src.to_str().unwrap(),
            "-filter_complex", &filter_complex,
            "-c:v", "copy",
            "-loglevel", "error",
            dst.to_str().unwrap()
        ])
    }
}
