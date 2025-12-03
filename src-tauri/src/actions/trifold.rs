use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct TrifoldAction;

impl VideoAction for TrifoldAction {
    fn id(&self) -> &'static str {
        "trifold"
    }

    fn execute(&self, src: &Path, out_dir: &Path, _config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "ab_tri")?;
        
        let filter_complex = "[0:v]split=3[a][b][c];[b]hflip[b_flip];[a][b_flip][c]hstack=inputs=3,scale=iw:ih";
        
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
