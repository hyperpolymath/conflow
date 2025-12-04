// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! Educational error messages
//!
//! Provides detailed, helpful explanations for common errors
//! to help users learn while fixing issues.

use std::path::Path;

/// Educational message with explanation and examples
#[derive(Debug, Clone)]
pub struct EducationalMessage {
    /// Short summary of the issue
    pub summary: String,
    /// Detailed explanation
    pub explanation: String,
    /// Example of correct usage
    pub example: Option<String>,
    /// Link to documentation
    pub docs_url: Option<String>,
}

impl EducationalMessage {
    /// Create a message for CUE constraint violation
    pub fn cue_constraint_violation(
        field: &str,
        expected: &str,
        got: &str,
    ) -> Self {
        Self {
            summary: format!("Constraint violation on field '{}'", field),
            explanation: format!(
                "The value '{}' does not satisfy the constraint '{}'.\n\n\
                 CUE uses unification to merge values with constraints.\n\
                 When a value doesn't match its constraint, validation fails.",
                got, expected
            ),
            example: Some(format!(
                "// Example constraint:\n\
                 {field}: int & >0 & <100\n\n\
                 // Valid:\n\
                 {field}: 42\n\n\
                 // Invalid:\n\
                 {field}: -1  // violates >0\n\
                 {field}: 150 // violates <100"
            )),
            docs_url: Some("https://cuelang.org/docs/tutorials/tour/types/bounds/".into()),
        }
    }

    /// Create a message for Nickel type error
    pub fn nickel_type_error(expected: &str, got: &str) -> Self {
        Self {
            summary: "Type mismatch".into(),
            explanation: format!(
                "Expected type '{}' but got '{}'.\n\n\
                 Nickel uses gradual typing: you can add type annotations\n\
                 to catch errors early, or leave them out for flexibility.",
                expected, got
            ),
            example: Some(
                "// With type annotation:\n\
                 let x : Number = 42\n\n\
                 // Type error:\n\
                 let x : Number = \"hello\"  // String != Number"
                    .into(),
            ),
            docs_url: Some("https://nickel-lang.org/user-manual/typing/".into()),
        }
    }

    /// Create a message for missing dependency
    pub fn missing_tool(tool: &str) -> Self {
        let (install_cmd, install_url) = match tool {
            "cue" => (
                "go install cuelang.org/go/cmd/cue@latest",
                "https://cuelang.org/docs/install/",
            ),
            "nickel" => (
                "cargo install nickel-lang-cli",
                "https://nickel-lang.org/getting-started/",
            ),
            _ => ("", ""),
        };

        Self {
            summary: format!("Tool '{}' not found", tool),
            explanation: format!(
                "conflow requires '{}' to be installed and available in your PATH.\n\n\
                 This tool is used for configuration {} and must be installed\n\
                 before running pipelines that use it.",
                tool,
                if tool == "cue" { "validation" } else { "generation" }
            ),
            example: if !install_cmd.is_empty() {
                Some(format!("# Install {}:\n{}", tool, install_cmd))
            } else {
                None
            },
            docs_url: if !install_url.is_empty() {
                Some(install_url.into())
            } else {
                None
            },
        }
    }

    /// Create a message explaining CUE vs Nickel choice
    pub fn tool_choice_explanation(file: &Path, recommended: &str) -> Self {
        let extension = file
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("unknown");

        let (explanation, alternative) = match recommended {
            "cue" => (
                "CUE is recommended because:\n\
                 • Your config uses constraint validation\n\
                 • No complex logic or functions detected\n\
                 • CUE's unification semantics fit well",
                "If you need to generate configs programmatically,\n\
                 consider using Nickel for generation + CUE for validation.",
            ),
            "nickel" => (
                "Nickel is recommended because:\n\
                 • Complex logic detected (functions, conditionals)\n\
                 • Config generation patterns found\n\
                 • Full programming language features needed",
                "After generating with Nickel, you can pipe\n\
                 the output to CUE for constraint validation.",
            ),
            _ => ("", ""),
        };

        Self {
            summary: format!("Recommendation: Use {} for {}", recommended, file.display()),
            explanation: explanation.into(),
            example: None,
            docs_url: Some("https://conflow.dev/docs/tool-selection/".into()),
        }
    }
}

impl std::fmt::Display for EducationalMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.summary)?;
        writeln!(f)?;
        writeln!(f, "{}", self.explanation)?;

        if let Some(ref example) = self.example {
            writeln!(f)?;
            writeln!(f, "Example:")?;
            writeln!(f, "────────")?;
            writeln!(f, "{}", example)?;
        }

        if let Some(ref url) = self.docs_url {
            writeln!(f)?;
            writeln!(f, "Learn more: {}", url)?;
        }

        Ok(())
    }
}
