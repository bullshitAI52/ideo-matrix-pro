use std::path::Path;
use anyhow::Result;
use uuid::Uuid;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct Md5Action;

impl VideoAction for Md5Action {
    fn id(&self) -> &'static str {
        "md5"
    }

    fn execute(&self, src: &Path, out_dir: &Path, _config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "md5")?;
        let uid = Uuid::new_v4().to_string();
        
        FFUtils::run(&[
            "-y",
            "-i", src.to_str().unwrap(),
            "-c", "copy",
            "-map_metadata", "-1",
            "-metadata", &format!("comment={}", uid),
            "-loglevel", "error",
            dst.to_str().unwrap()
        ])
    }
}
