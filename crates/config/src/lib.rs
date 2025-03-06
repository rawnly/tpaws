mod global_config;
mod project_config;
pub mod util;

pub use global_config::*;
pub use project_config::*;

pub const DEFAULT_AI_MODEL: &str = "llama3-8b-8192";
