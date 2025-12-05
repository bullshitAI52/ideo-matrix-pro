use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct PitchAction;

impl VideoAction for PitchAction {
    fn id(&self) -> &'static str {
        "pitch"
    }

    fn execute(&self, src: &Path, out_dir: &Path, config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "pitch")?;
        
        let range = config.params.get("pitch_range").and_then(|v| v.as_f64()).unwrap_or(0.5);
        let mut rng = rand::thread_rng();
        use rand::Rng;
        let semitones = rng.gen_range(-range..range);
        
        // Convert semitones to rate multiplier: 2^(semitones/12)
        let rate_mult = 2.0_f64.powf(semitones / 12.0);
        let new_rate = 44100.0 * rate_mult;
        
        let af = format!("asetrate={},aresample=44100", new_rate);
        
        FFUtils::run(&[
            "-y",
            "-i", src.to_str().unwrap(),
            "-af", &af,
            "-c:v", "copy",
            "-loglevel", "error",
            dst.to_str().unwrap()
        ])
    }
}
