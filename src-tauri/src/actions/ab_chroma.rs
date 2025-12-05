use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct AbChromaAction;

impl VideoAction for AbChromaAction {
    fn id(&self) -> &'static str {
        "ab_chroma"
    }

    fn execute(&self, src: &Path, out_dir: &Path, _config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "ab_chroma")?;
        
        // Chromatic aberration effect using chromashift (works on YUV, efficient and robust)
        // cb/cr shift values create the color fringe
        let filter_complex = "chromashift=cb=4:cr=-4:edge=smear";
        
        FFUtils::run(&[
            "-y",
            "-i", src.to_str().unwrap(),
            "-filter_complex", filter_complex,
            "-c:a", "copy",
            "-loglevel", "error",
            dst.to_str().unwrap()
        ])
    }
}
