use std::path::Path;
use anyhow::Result;
use crate::core::{VideoAction, ActionConfig, FFUtils};

// Placeholder implementations for material overlay actions
pub struct StickerAction;
pub struct MaskAction;
pub struct PipAction;
pub struct EdgeEffectAction;
pub struct LightEffectAction;
pub struct GoodsTemplateAction;

impl VideoAction for StickerAction {
    fn id(&self) -> &'static str { "sticker" }
    fn execute(&self, src: &Path, out_dir: &Path, config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "sticker")?;
        
        if let Some(path) = &config.sticker_path {
            // Sticker centered
            let escaped_path = FFUtils::escape_path(path);
            let vf = format!("movie='{}'[s];[in][s]overlay=(W-w)/2:(H-h)/2", escaped_path);
            FFUtils::run(&["-y", "-i", src.to_str().unwrap(), "-vf", &vf, "-c:a", "copy", "-loglevel", "error", dst.to_str().unwrap()])
        } else {
            // Fallback
            FFUtils::run(&["-y", "-i", src.to_str().unwrap(), "-c", "copy", "-loglevel", "error", dst.to_str().unwrap()])
        }
    }
}

impl VideoAction for MaskAction {
    fn id(&self) -> &'static str { "mask" }
    fn execute(&self, src: &Path, out_dir: &Path, config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "mask")?;
        
        if let Some(path) = &config.mask_path {
            // Mask overlay (full stretch or centered) - here we assume overlay
            let escaped_path = FFUtils::escape_path(path);
            let vf = format!("movie='{}'[m];[in][m]overlay=0:0", escaped_path);
            FFUtils::run(&["-y", "-i", src.to_str().unwrap(), "-vf", &vf, "-c:a", "copy", "-loglevel", "error", dst.to_str().unwrap()])
        } else {
            // Fallback
            FFUtils::run(&["-y", "-i", src.to_str().unwrap(), "-c", "copy", "-loglevel", "error", dst.to_str().unwrap()])
        }
    }
}

impl VideoAction for PipAction {
    fn id(&self) -> &'static str { "pip" }
    fn execute(&self, src: &Path, out_dir: &Path, config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "pip")?;
        
        if let Some(path) = &config.pip_path {
            // Picture-in-Picture: overlay video in bottom-right corner, scaled to 25%
            let escaped_path = FFUtils::escape_path(path);
            let vf = format!("movie='{}',scale=iw*0.25:ih*0.25[pip];[in][pip]overlay=W-w-10:H-h-10", escaped_path);
            FFUtils::run(&["-y", "-i", src.to_str().unwrap(), "-vf", &vf, "-c:a", "copy", "-loglevel", "error", dst.to_str().unwrap()])
        } else {
            // Fallback
            FFUtils::run(&["-y", "-i", src.to_str().unwrap(), "-c", "copy", "-loglevel", "error", dst.to_str().unwrap()])
        }
    }
}

impl VideoAction for EdgeEffectAction {
    fn id(&self) -> &'static str { "edge_effect" }
    fn execute(&self, src: &Path, out_dir: &Path, _config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "edge")?;
        let vf = "edgedetect=mode=colormix";
        FFUtils::run(&["-y", "-i", src.to_str().unwrap(), "-vf", vf, "-c:a", "copy", "-loglevel", "error", dst.to_str().unwrap()])
    }
}

impl VideoAction for LightEffectAction {
    fn id(&self) -> &'static str { "light_effect" }
    fn execute(&self, src: &Path, out_dir: &Path, config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "light")?;
        
        if let Some(path) = &config.light_effect_path {
            // Light effect overlay: blend mode for additive light effect
            let escaped_path = FFUtils::escape_path(path);
            let vf = format!("movie='{}'[light];[in][light]overlay=0:0:format=auto", escaped_path);
            FFUtils::run(&["-y", "-i", src.to_str().unwrap(), "-vf", &vf, "-c:a", "copy", "-loglevel", "error", dst.to_str().unwrap()])
        } else {
            // Fallback: add brightness/glow effect
            let vf = "eq=brightness=0.1:contrast=1.1";
            FFUtils::run(&["-y", "-i", src.to_str().unwrap(), "-vf", vf, "-c:a", "copy", "-loglevel", "error", dst.to_str().unwrap()])
        }
    }
}

impl VideoAction for GoodsTemplateAction {
    fn id(&self) -> &'static str { "goods_template" }
    fn execute(&self, src: &Path, out_dir: &Path, config: &ActionConfig) -> Result<()> {
        let dst = FFUtils::get_dst(src, out_dir, "goods")?;
        
        if let Some(path) = &config.goods_path {
            // Goods template: overlay template on top (assuming template has transparency)
            let escaped_path = FFUtils::escape_path(path);
            let vf = format!("movie='{}'[template];[in][template]overlay=0:0", escaped_path);
            FFUtils::run(&["-y", "-i", src.to_str().unwrap(), "-vf", &vf, "-c:a", "copy", "-loglevel", "error", dst.to_str().unwrap()])
        } else {
            // Fallback
            FFUtils::run(&["-y", "-i", src.to_str().unwrap(), "-c", "copy", "-loglevel", "error", dst.to_str().unwrap()])
        }
    }
}
