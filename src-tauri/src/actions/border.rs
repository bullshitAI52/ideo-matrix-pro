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
            let width = config.params.get("border_width").and_then(|v| v.as_i64()).unwrap_or(20) as f64;
            
            // Calculate scale factor: (W - 2*width) / W
            // Since we don't know W here easily without probing, we use a relative approach or assume standard width
            // Better approach: scale=iw-2*width:ih-2*width
            // But for simplicity and safety, let's use relative scaling assuming 1080p roughly, or just use padding
            // Let's use padding approach: scale input to (W-2*w):(H-2*w) then overlay on blurred background
            
            // Simplified approach: boxblur background, then overlay scaled foreground
            // We use an expression for scale: iw-2*{width}:ih-2*{width}
            // Note: This might fail if width is too large.
            
            let filter_complex = format!(
                "[0:v]split=2[bg][fg];[bg]scale=iw:ih,boxblur=20[bg_b];[fg]scale=iw-2*{0}:ih-2*{0}[fg_s];[bg_b][fg_s]overlay={0}:{0}",
                width
            );
            
            FFUtils::run(&[
                "-y",
                "-i", src.to_str().unwrap(),
                "-filter_complex", &filter_complex,
                "-c:a", "copy",
                "-loglevel", "error",
                dst.to_str().unwrap()
            ])
        }
    }
}
