use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct CornerAction;

impl VideoAction for CornerAction {
    fn id(&self) -> &'static str {
        "corner"
    }

    fn execute(&self, src: &Path, out_dir: &Path, _config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "corner")?;
        
        let filter_complex = "[0:v]split=2[main][blur];[blur]crop=iw/4:ih/4:0:0,boxblur=10[blur1];[blur]crop=iw/4:ih/4:iw*3/4:0,boxblur=10[blur2];[blur]crop=iw/4:ih/4:0:ih*3/4,boxblur=10[blur3];[blur]crop=iw/4:ih/4:iw*3/4:ih*3/4,boxblur=10[blur4];[main][blur1]overlay=0:0[tmp1];[tmp1][blur2]overlay=iw*3/4:0[tmp2];[tmp2][blur3]overlay=0:ih*3/4[tmp3];[tmp3][blur4]overlay=iw*3/4:ih*3/4";
        
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
