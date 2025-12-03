use std::path::Path;
use anyhow::Result;
use rand::Rng;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct CropAction;

impl VideoAction for CropAction {
    fn id(&self) -> &'static str {
        "crop"
    }

    fn execute(&self, src: &Path, out_dir: &Path, _config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "crop")?;
        
        let mut rng = rand::thread_rng();
        let ratio: f64 = rng.gen_range(0.95..0.98);
        
        let vf = format!("crop=iw*{:.3}:ih*{:.3}:(iw-ow)/2:(ih-oh)/2", ratio, ratio);
        
        FFUtils::run(&[
            "-y",
            "-i", src.to_str().unwrap(),
            "-vf", &vf,
            "-c:a", "copy",
            "-loglevel", "error",
            dst.to_str().unwrap()
        ])
    }
}
