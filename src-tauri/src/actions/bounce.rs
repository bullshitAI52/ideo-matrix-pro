use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct BounceAction;

impl VideoAction for BounceAction {
    fn id(&self) -> &'static str {
        "bounce"
    }

    fn execute(&self, src: &Path, out_dir: &Path, _config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "bounce")?;
        
        let filter_complex = "[0:v]split=2[bg][fg];[bg]scale=iw:ih,boxblur=20[bg_blur];[fg]scale=iw*0.85:ih*0.85[fg_s];[bg_blur][fg_s]overlay=x='(W-w)/2+20*sin(t)':y='(H-h)/2+10*cos(t*1.5)'";
        
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
