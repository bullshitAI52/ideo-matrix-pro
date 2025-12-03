pub mod ffutils;
pub mod factory;

pub use ffutils::FFUtils;
pub use factory::ActionFactory;

use std::path::Path;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionConfig {
    // Generic config map for flexibility
    #[serde(flatten)]
    pub params: serde_json::Value,
}

impl Default for ActionConfig {
    fn default() -> Self {
        Self {
            params: serde_json::json!({}),
        }
    }
}

/// Trait that all video processing actions must implement
pub trait VideoAction: Send + Sync {
    /// Execute the action on the source file
    fn execute(&self, src: &Path, out_dir: &Path, config: &ActionConfig) -> Result<()>;
    
    /// Get the identifier for this action (e.g., "crop", "speed")
    fn id(&self) -> &'static str;
}
