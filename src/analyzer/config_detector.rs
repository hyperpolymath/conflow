// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! Configuration format detection

use std::path::Path;

use crate::errors::ConflowError;

/// Detected configuration format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    Json,
    Yaml,
    Toml,
    Cue,
    Nickel,
    Unknown,
}

/// Detect the format of a configuration file
pub fn detect_format(content: &str, path: &Path) -> Result<ConfigFormat, ConflowError> {
    // First try extension-based detection
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        match ext.to_lowercase().as_str() {
            "json" => return Ok(ConfigFormat::Json),
            "yaml" | "yml" => return Ok(ConfigFormat::Yaml),
            "toml" => return Ok(ConfigFormat::Toml),
            "cue" => return Ok(ConfigFormat::Cue),
            "ncl" => return Ok(ConfigFormat::Nickel),
            _ => {}
        }
    }

    // Try content-based detection
    let trimmed = content.trim();

    // JSON detection
    if (trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']'))
    {
        if serde_json::from_str::<serde_json::Value>(content).is_ok() {
            return Ok(ConfigFormat::Json);
        }
    }

    // TOML detection (look for = assignments and [sections])
    if trimmed.contains(" = ") || trimmed.contains("\n[") {
        if toml::from_str::<toml::Value>(content).is_ok() {
            return Ok(ConfigFormat::Toml);
        }
    }

    // YAML detection
    if serde_yaml::from_str::<serde_yaml::Value>(content).is_ok() {
        // Could be YAML (most formats are valid YAML)
        // Check for YAML-specific patterns
        if trimmed.contains(": ") || trimmed.starts_with("---") || trimmed.contains("\n- ") {
            return Ok(ConfigFormat::Yaml);
        }
    }

    // CUE detection (look for CUE-specific syntax)
    if content.contains("#") && content.contains(":") {
        // Likely CUE with definitions
        return Ok(ConfigFormat::Cue);
    }

    // Nickel detection (look for Nickel-specific syntax)
    if content.contains(" = ") && (content.contains("let ") || content.contains("fun ")) {
        return Ok(ConfigFormat::Nickel);
    }

    Ok(ConfigFormat::Unknown)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_detect_json() {
        let content = r#"{"name": "test", "value": 42}"#;
        let format = detect_format(content, &PathBuf::from("config.json")).unwrap();
        assert_eq!(format, ConfigFormat::Json);
    }

    #[test]
    fn test_detect_yaml() {
        let content = "name: test\nvalue: 42";
        let format = detect_format(content, &PathBuf::from("config.yaml")).unwrap();
        assert_eq!(format, ConfigFormat::Yaml);
    }

    #[test]
    fn test_detect_toml() {
        let content = "[package]\nname = \"test\"";
        let format = detect_format(content, &PathBuf::from("config.toml")).unwrap();
        assert_eq!(format, ConfigFormat::Toml);
    }

    #[test]
    fn test_detect_cue_by_extension() {
        let content = "#Config: { name: string }";
        let format = detect_format(content, &PathBuf::from("schema.cue")).unwrap();
        assert_eq!(format, ConfigFormat::Cue);
    }

    #[test]
    fn test_detect_nickel_by_extension() {
        let content = "{ name = \"test\" }";
        let format = detect_format(content, &PathBuf::from("config.ncl")).unwrap();
        assert_eq!(format, ConfigFormat::Nickel);
    }
}
