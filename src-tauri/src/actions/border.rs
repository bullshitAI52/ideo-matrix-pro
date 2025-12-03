use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct BorderAction;

impl VideoAction for BorderAction {
    fn id(&self) -> &'static str {
        "border"
    }

    fn execute(&self, src: &Path, out_dir: &Path, _config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "border")?;
        
        // Default to blur border for now
        // [0:v]split=2[bg][fg];[bg]scale=iw:ih,boxblur=20[bg_b];[fg]scale=iw*0.9:ih*0.9[fg_s];[bg_b][fg_s]overlay=(W-w)/2:(H-h)/2
        let filter_complex = "[0:v]split=2[bg][fg];[bg]scale=iw:ih,boxblur=20[bg_b];[fg]scale=iw*0.9:ih*0.9[fg_s];[bg_b][fg_s]overlay=(W-w)/2:(H-h)/2";
        
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
