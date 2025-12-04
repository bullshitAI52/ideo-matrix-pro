use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct BorderAction;

impl VideoAction for BorderAction {
    fn id(&self) -> &'static str {
        "border"
    }

    fn execute(&self, src: &Path, out_dir: &Path, config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "border")?;
        
        if let Some(path) = &config.border_path {
            // Use custom border image - overlay it on top
            let vf = format!("movie='{}'[border];[in][border]overlay=0:0", path);
            FFUtils::run(&[
                "-y",
                "-i", src.to_str().unwrap(),
                "-vf", &vf,
                "-c:a", "copy",
                "-loglevel", "error",
                dst.to_str().unwrap()
            ])
        } else {
            // Default: blur border effect
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
}
