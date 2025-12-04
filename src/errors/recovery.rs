// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! Error recovery suggestions
//!
//! Provides actionable suggestions for recovering from errors.

/// A recovery suggestion with concrete steps
#[derive(Debug, Clone)]
pub struct RecoverySuggestion {
    /// Brief description of what to do
    pub action: String,
    /// Detailed steps
    pub steps: Vec<String>,
    /// Commands to run
    pub commands: Vec<String>,
    /// Whether this is an automated fix
    pub auto_fixable: bool,
}

impl RecoverySuggestion {
    /// Suggest installing a missing tool
    pub fn install_tool(tool: &str) -> Self {
        match tool {
            "cue" => Self {
                action: "Install CUE".into(),
                steps: vec![
                    "CUE is required for configuration validation".into(),
                    "Choose an installation method below".into(),
                ],
                commands: vec![
                    "# Using Go:".into(),
                    "go install cuelang.org/go/cmd/cue@latest".into(),
                    "".into(),
                    "# Using Homebrew (macOS/Linux):".into(),
                    "brew install cue-lang/tap/cue".into(),
                    "".into(),
                    "# Using Nix:".into(),
                    "nix-env -iA nixpkgs.cue".into(),
                ],
                auto_fixable: false,
            },
            "nickel" => Self {
                action: "Install Nickel".into(),
                steps: vec![
                    "Nickel is required for configuration generation".into(),
                    "Choose an installation method below".into(),
                ],
                commands: vec![
                    "# Using Cargo:".into(),
                    "cargo install nickel-lang-cli".into(),
                    "".into(),
                    "# Using Nix:".into(),
                    "nix-env -iA nixpkgs.nickel".into(),
                ],
                auto_fixable: false,
            },
            _ => Self {
                action: format!("Install {}", tool),
                steps: vec![format!("Install {} and ensure it's in your PATH", tool)],
                commands: vec![],
                auto_fixable: false,
            },
        }
    }

    /// Suggest fixing a circular dependency
    pub fn fix_circular_dependency(stages: &[String]) -> Self {
        Self {
            action: "Remove circular dependency".into(),
            steps: vec![
                format!("Detected cycle: {}", stages.join(" → ")),
                "Review your stage dependencies".into(),
                "Ensure stages form a directed acyclic graph (DAG)".into(),
            ],
            commands: vec![
                "# Visualize your pipeline:".into(),
                "conflow graph --format mermaid".into(),
            ],
            auto_fixable: false,
        }
    }

    /// Suggest creating a pipeline file
    pub fn create_pipeline() -> Self {
        Self {
            action: "Create a pipeline configuration".into(),
            steps: vec![
                "No .conflow.yaml found in current directory".into(),
                "Initialize a new project or create the file manually".into(),
            ],
            commands: vec![
                "# Initialize with wizard:".into(),
                "conflow init".into(),
                "".into(),
                "# Or use a template:".into(),
                "conflow init --template cue-validation".into(),
            ],
            auto_fixable: true,
        }
    }

    /// Suggest fixing invalid YAML
    pub fn fix_yaml_syntax(line: Option<usize>, column: Option<usize>) -> Self {
        let location = match (line, column) {
            (Some(l), Some(c)) => format!(" at line {}, column {}", l, c),
            (Some(l), None) => format!(" at line {}", l),
            _ => String::new(),
        };

        Self {
            action: format!("Fix YAML syntax error{}", location),
            steps: vec![
                "Check for common YAML issues:".into(),
                "  • Incorrect indentation (use spaces, not tabs)".into(),
                "  • Missing colons after keys".into(),
                "  • Unquoted special characters".into(),
                "  • Incorrect list formatting".into(),
            ],
            commands: vec![
                "# Validate your YAML:".into(),
                "python -c \"import yaml; yaml.safe_load(open('.conflow.yaml'))\"".into(),
            ],
            auto_fixable: false,
        }
    }

    /// Suggest fixing missing input files
    pub fn fix_missing_inputs(pattern: &str, stage: &str) -> Self {
        Self {
            action: format!("Add files matching pattern '{}'", pattern),
            steps: vec![
                format!("Stage '{}' expects input files matching: {}", stage, pattern),
                "Either create the files or update the pattern".into(),
            ],
            commands: vec![
                "# List files that would match:".into(),
                format!("ls -la {}", pattern.replace('*', "\\*")),
            ],
            auto_fixable: false,
        }
    }
}

impl std::fmt::Display for RecoverySuggestion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "→ {}", self.action)?;

        for step in &self.steps {
            writeln!(f, "  {}", step)?;
        }

        if !self.commands.is_empty() {
            writeln!(f)?;
            for cmd in &self.commands {
                writeln!(f, "  {}", cmd)?;
            }
        }

        Ok(())
    }
}
