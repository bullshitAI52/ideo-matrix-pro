use std::path::Path;
use anyhow::Result;
use rand::Rng;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct CropAction;

impl VideoAction for CropAction {
    fn id(&self) -> &'static str {
        "crop"
    }

    fn execute(&self, src: &Path, out_dir: &Path, config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "crop")?;
        
        let mut rng = rand::thread_rng();
        
        // Get parameters from config or use defaults
        let min_crop = config.params.get("crop_min").and_then(|v| v.as_f64()).unwrap_or(0.01);
        let max_crop = config.params.get("crop_max").and_then(|v| v.as_f64()).unwrap_or(0.05);
        
        // Calculate keep ratio (e.g., crop 5% means keep 95%)
        let crop_amount = rng.gen_range(min_crop..max_crop);
        let ratio = 1.0 - crop_amount;
        
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
