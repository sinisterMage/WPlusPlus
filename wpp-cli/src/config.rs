use anyhow::{Context, Result};
use std::path::Path;

/// Functional configuration parsed from `wpp.config.hs`
#[derive(Debug, Default)]
pub struct WppConfig {
    pub project_name: Option<String>,
    pub entrypoint: Option<String>,
    pub version: Option<String>,
    pub license: Option<String>,
    pub author: Option<String>,
    pub flags: Vec<String>,
    pub messages: Vec<String>,
}

impl WppConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let content =
            std::fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
        let mut cfg = Self::default();

        for raw in content.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with("--") {
                continue;
            }

            // match lines like  key "value"
            if let Some(value) = line.split('"').nth(1) {
                match () {
                    _ if line.starts_with("entrypoint") => cfg.entrypoint = Some(value.to_string()),
                    _ if line.starts_with("projectName") || line.starts_with("package") => {
                        cfg.project_name = Some(value.to_string())
                    }
                    _ if line.starts_with("version") => cfg.version = Some(value.to_string()),
                    _ if line.starts_with("license") => cfg.license = Some(value.to_string()),
                    _ if line.starts_with("author") => cfg.author = Some(value.to_string()),
                    _ if line.starts_with("flag") => cfg.flags.push(value.to_string()),
                    _ if line.starts_with("println") => cfg.messages.push(value.to_string()),
                    _ => {}
                }
            } else if line.starts_with("flags") {
                // support flags ["--emit-ir", "--opt=2"]
                let inside = line.split('[').nth(1).unwrap_or("").split(']').next().unwrap_or("");
                for p in inside.split(',') {
                    let trimmed = p.trim().trim_matches('"');
                    if !trimmed.is_empty() {
                        cfg.flags.push(trimmed.to_string());
                    }
                }
            }
        }

        Ok(cfg)
    }
}
