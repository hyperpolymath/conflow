// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! Pattern recognition for configuration files
//!
//! Identifies common patterns in configuration files to aid in tool selection.

/// Common configuration patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigPattern {
    /// Simple key-value pairs
    SimpleKeyValue,
    /// Hierarchical/nested structure
    Hierarchical,
    /// List/array of similar items
    HomogeneousList,
    /// Environment-based variants
    EnvironmentConfig,
    /// Schema/type definitions
    SchemaDefinition,
    /// Template with variables
    Templated,
    /// Resource definitions (K8s, Terraform, etc.)
    ResourceDefinition,
}

/// Detect patterns in configuration content
pub fn detect_patterns(content: &str) -> Vec<ConfigPattern> {
    let mut patterns = Vec::new();

    // Check for simple key-value (shallow structure)
    if is_simple_key_value(content) {
        patterns.push(ConfigPattern::SimpleKeyValue);
    }

    // Check for hierarchical structure
    if is_hierarchical(content) {
        patterns.push(ConfigPattern::Hierarchical);
    }

    // Check for homogeneous lists
    if has_homogeneous_lists(content) {
        patterns.push(ConfigPattern::HomogeneousList);
    }

    // Check for environment config patterns
    if is_environment_config(content) {
        patterns.push(ConfigPattern::EnvironmentConfig);
    }

    // Check for schema definitions
    if is_schema_definition(content) {
        patterns.push(ConfigPattern::SchemaDefinition);
    }

    // Check for templating
    if is_templated(content) {
        patterns.push(ConfigPattern::Templated);
    }

    // Check for resource definitions
    if is_resource_definition(content) {
        patterns.push(ConfigPattern::ResourceDefinition);
    }

    patterns
}

fn is_simple_key_value(content: &str) -> bool {
    // Simple if nesting depth is low
    let max_depth = calculate_depth(content);
    max_depth <= 2
}

fn is_hierarchical(content: &str) -> bool {
    let max_depth = calculate_depth(content);
    max_depth > 2
}

fn has_homogeneous_lists(content: &str) -> bool {
    // Look for array patterns with similar structures
    content.contains("[") && content.matches('[').count() > 1
}

fn is_environment_config(content: &str) -> bool {
    let env_keywords = ["dev", "staging", "prod", "production", "test", "development"];
    env_keywords.iter().any(|k| content.to_lowercase().contains(k))
}

fn is_schema_definition(content: &str) -> bool {
    // Look for type definition patterns
    let schema_patterns = [
        "#",         // CUE definitions
        "$schema",   // JSON Schema
        "type:",     // Various
        "properties:", // JSON Schema
        "required:", // JSON Schema
    ];
    schema_patterns.iter().any(|p| content.contains(p))
}

fn is_templated(content: &str) -> bool {
    // Look for template variable patterns
    let template_patterns = [
        "{{", "}}",           // Mustache/Handlebars
        "${", "}",            // Shell/Terraform style
        "<%", "%>",           // ERB style
        "[[", "]]",           // Alternative brackets
    ];

    template_patterns.chunks(2).any(|pair| {
        content.contains(pair[0]) && content.contains(pair[1])
    })
}

fn is_resource_definition(content: &str) -> bool {
    let resource_patterns = [
        "apiVersion:", "kind:", // Kubernetes
        "resource \"", "provider \"", // Terraform
        "AWSTemplateFormatVersion", // CloudFormation
    ];
    resource_patterns.iter().any(|p| content.contains(p))
}

fn calculate_depth(content: &str) -> usize {
    let mut max_depth: usize = 0;
    let mut current_depth: usize = 0;

    for ch in content.chars() {
        match ch {
            '{' | '[' => {
                current_depth += 1;
                max_depth = max_depth.max(current_depth);
            }
            '}' | ']' => {
                current_depth = current_depth.saturating_sub(1);
            }
            _ => {}
        }
    }

    max_depth
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_key_value() {
        let content = r#"{"name": "test", "value": 42}"#;
        let patterns = detect_patterns(content);
        assert!(patterns.contains(&ConfigPattern::SimpleKeyValue));
    }

    #[test]
    fn test_hierarchical() {
        let content = r#"{"a": {"b": {"c": {"d": 1}}}}"#;
        let patterns = detect_patterns(content);
        assert!(patterns.contains(&ConfigPattern::Hierarchical));
    }

    #[test]
    fn test_environment_config() {
        let content = r#"{"environment": "production", "debug": false}"#;
        let patterns = detect_patterns(content);
        assert!(patterns.contains(&ConfigPattern::EnvironmentConfig));
    }

    #[test]
    fn test_kubernetes_resource() {
        let content = r#"apiVersion: v1
kind: ConfigMap
metadata:
  name: test"#;
        let patterns = detect_patterns(content);
        assert!(patterns.contains(&ConfigPattern::ResourceDefinition));
    }
}
