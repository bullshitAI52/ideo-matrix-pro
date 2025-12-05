use std::path::Path;
use anyhow::Result;
use rand::Rng;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct SpeedAction;

impl VideoAction for SpeedAction {
    fn id(&self) -> &'static str {
        "speed"
    }

    fn execute(&self, src: &Path, out_dir: &Path, config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "spd")?;
        
        let mut rng = rand::thread_rng();
        let range = config.params.get("speed_range").and_then(|v| v.as_f64()).unwrap_or(0.05);
        let speed: f64 = rng.gen_range((1.0 - range)..(1.0 + range));
        
        let setpts = format!("setpts={:.4}*PTS", 1.0/speed);
        let atempo = format!("atempo={:.4}", speed);
        
        FFUtils::run(&[
            "-y",
            "-i", src.to_str().unwrap(),
            "-filter:v", &setpts,
            "-filter:a", &atempo,
            "-loglevel", "error",
            dst.to_str().unwrap()
        ])
    }
}
