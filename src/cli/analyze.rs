// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! Analyze command - analyze configuration files and recommend tools

use colored::Colorize;
use miette::Result;
use std::path::PathBuf;

use super::OutputFormat;
use crate::analyzer::ConfigAnalyzer;

/// Run the analyze command
pub async fn run(files: Vec<PathBuf>, format: OutputFormat, verbose: bool) -> Result<()> {
    if files.is_empty() {
        return Err(miette::miette!(
            "No files specified.\n\n\
             Usage: conflow analyze <file>..."
        ));
    }

    let analyzer = ConfigAnalyzer::new();

    for file in &files {
        if !file.exists() {
            eprintln!("{}: File not found: {}", "Warning".yellow(), file.display());
            continue;
        }

        match analyzer.analyze(file).await {
            Ok(analysis) => {
                match format {
                    OutputFormat::Text => print_text_analysis(file, &analysis, verbose),
                    OutputFormat::Json => print_json_analysis(file, &analysis)?,
                }
            }
            Err(e) => {
                eprintln!("{}: Failed to analyze {}: {}", "Error".red(), file.display(), e);
            }
        }
    }

    Ok(())
}

fn print_text_analysis(
    file: &PathBuf,
    analysis: &crate::analyzer::Analysis,
    verbose: bool,
) {
    println!();
    println!("{}: {}", "Analyzing".bold(), file.display());
    println!("{}", "═".repeat(50));
    println!();

    // Format info
    println!("{}:  {:?}", "Format".bold(), analysis.format);
    println!(
        "{}:    {} lines",
        "Size".bold(),
        analysis.complexity.line_count
    );
    println!();

    // Complexity analysis
    println!("{}:", "Complexity Analysis".bold());
    print_check("Logic/conditionals", analysis.complexity.has_logic);
    print_check("Functions", analysis.complexity.has_functions);
    print_check("Constraints", analysis.complexity.has_constraints);
    print_check("Nested structures", analysis.complexity.nesting_depth > 2);
    println!(
        "  Nesting depth: {}",
        analysis.complexity.nesting_depth
    );
    println!();

    // Recommendation
    println!(
        "{}: Use {}",
        "Recommendation".bold(),
        format!("{:?}", analysis.recommendation.primary).green().bold()
    );
    println!("{}", "═".repeat(50));
    println!();

    println!("{}:", "Why?".bold());
    for reason in &analysis.recommendation.rationale {
        println!("  • {}", reason);
    }

    if !analysis.recommendation.alternatives.is_empty() {
        println!();
        println!("{}:", "Alternatives".bold());
        for alt in &analysis.recommendation.alternatives {
            println!("  • {:?}: {}", alt.tool, alt.reason);
        }
    }

    if let Some(ref combined) = analysis.recommendation.combined_approach {
        println!();
        println!("{}: {}", "Combined approach".bold(), combined);
    }

    if verbose {
        println!();
        println!("{}:", "Example".bold());
        println!("{}", "─".repeat(40));

        match analysis.recommendation.primary {
            crate::analyzer::RecommendedTool::Cue => {
                println!(
                    r#"// schema.cue
#Config: {{
    name:     string
    replicas: int & >=1 & <=10
    port:     int & >=1 & <=65535
}}"#
                );
            }
            crate::analyzer::RecommendedTool::Nickel => {
                println!(
                    r#"# config.ncl
{{
  name = "my-app",
  replicas =
    let env = "prod" in
    if env == "prod" then 5 else 1,
  port = 8080,
}}"#
                );
            }
        }
    }

    println!();
    println!("{}:", "Next steps".bold());
    println!("  1. {}", "conflow init --template <template>".cyan());
    println!("  2. Add your configuration files");
    println!("  3. {}", "conflow run".cyan());
    println!();
}

fn print_json_analysis(
    file: &PathBuf,
    analysis: &crate::analyzer::Analysis,
) -> Result<()> {
    let json = serde_json::json!({
        "file": file.display().to_string(),
        "format": format!("{:?}", analysis.format),
        "complexity": {
            "has_logic": analysis.complexity.has_logic,
            "has_functions": analysis.complexity.has_functions,
            "has_constraints": analysis.complexity.has_constraints,
            "line_count": analysis.complexity.line_count,
            "nesting_depth": analysis.complexity.nesting_depth,
        },
        "recommendation": {
            "primary": format!("{:?}", analysis.recommendation.primary),
            "rationale": analysis.recommendation.rationale,
            "alternatives": analysis.recommendation.alternatives.iter().map(|a| {
                serde_json::json!({
                    "tool": format!("{:?}", a.tool),
                    "reason": a.reason,
                })
            }).collect::<Vec<_>>(),
            "combined_approach": analysis.recommendation.combined_approach,
        }
    });

    println!("{}", serde_json::to_string_pretty(&json).map_err(|e| {
        miette::miette!("Failed to serialize JSON: {}", e)
    })?);

    Ok(())
}

fn print_check(label: &str, value: bool) {
    let (icon, status) = if value {
        ("✓".green(), "Detected")
    } else {
        ("✗".dimmed(), "Not detected")
    };
    println!("  {} {}: {}", icon, label, status.dimmed());
}
