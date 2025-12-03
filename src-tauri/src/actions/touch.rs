use std::path::Path;
use std::fs;
use std::time::SystemTime;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

pub struct TouchAction;

impl VideoAction for TouchAction {
    fn id(&self) -> &'static str {
        "touch"
    }

    fn execute(&self, src: &Path, out_dir: &Path, _config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "touch")?;
        
        // Copy file
        fs::copy(src, &dst)?;
        
        // Update timestamp
        let now = SystemTime::now();
        filetime::set_file_mtime(&dst, filetime::FileTime::from_system_time(now))?;
        
        Ok(())
    }
}
