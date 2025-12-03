use std::path::Path;
use anyhow::Result;
use rand::Rng;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct EncodeAction;

impl VideoAction for EncodeAction {
    fn id(&self) -> &'static str {
        "encode"
    }

    fn execute(&self, src: &Path, out_dir: &Path, _config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "encode")?;
        
        let mut rng = rand::thread_rng();
        let crf = rng.gen_range(18..=28);
        
        let presets = ["ultrafast", "superfast", "veryfast", "faster", "fast", "medium"];
        let preset = presets[rng.gen_range(0..presets.len())];
        
        FFUtils::run(&[
            "-y",
            "-i", src.to_str().unwrap(),
            "-c:v", "libx264",
            "-crf", &crf.to_string(),
            "-preset", preset,
            "-c:a", "copy",
            "-loglevel", "error",
            dst.to_str().unwrap()
        ])
    }
}
