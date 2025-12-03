use std::path::Path;
use anyhow::Result;
use rand::Rng;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct RotateAction;

impl VideoAction for RotateAction {
    fn id(&self) -> &'static str {
        "rotate"
    }

    fn execute(&self, src: &Path, out_dir: &Path, _config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "rot")?;
        
        let mut rng = rand::thread_rng();
        let degree: f64 = rng.gen_range(-1.5..1.5);
        
        // rotate={degree}*PI/180,scale=1.02*iw:-1
        let vf = format!("rotate={}*PI/180,scale=1.02*iw:-1", degree);
        
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
