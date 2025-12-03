use std::path::Path;
use anyhow::Result;
use rand::Rng;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct ColorAction;

impl VideoAction for ColorAction {
    fn id(&self) -> &'static str {
        "color"
    }

    fn execute(&self, src: &Path, out_dir: &Path, _config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "color")?;
        
        let mut rng = rand::thread_rng();
        let val: f64 = rng.gen_range(0.05..0.12);
        
        let vf = if rng.gen_bool(0.5) {
            format!("eq=gamma_r={:.4}:gamma_b={:.4}:saturation=1.1", 1.0+val, 1.0-val)
        } else {
            format!("eq=gamma_r={:.4}:gamma_b={:.4}:saturation=1.05", 1.0-val, 1.0+val)
        };
        
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
