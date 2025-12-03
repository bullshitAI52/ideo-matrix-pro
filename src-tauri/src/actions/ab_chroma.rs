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
        
        // Chromatic aberration effect using split and shift
        let filter_complex = "[0:v]split=3[r][g][b];[r]lutrgb=r=val:g=0:b=0[r_only];[g]lutrgb=r=0:g=val:b=0,crop=iw:ih:2:0[g_shift];[b]lutrgb=r=0:g=0:b=val,crop=iw:ih:-2:0[b_shift];[r_only][g_shift]blend=all_mode=addition[rg];[rg][b_shift]blend=all_mode=addition";
        
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
