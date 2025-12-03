use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct ZoomAction;

impl VideoAction for ZoomAction {
    fn id(&self) -> &'static str {
        "zoom"
    }

    fn execute(&self, src: &Path, out_dir: &Path, _config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "ai_zoom")?;
        
        let vf = "zoompan=z='min(zoom+0.0015,1.2)':d=700:x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)'";
        
        FFUtils::run(&[
            "-y",
            "-i", src.to_str().unwrap(),
            "-vf", vf,
            "-c:a", "copy",
            "-loglevel", "error",
            dst.to_str().unwrap()
        ])
    }
}
