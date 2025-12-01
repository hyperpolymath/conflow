//! RSR command - RSR integration and compliance checking

use colored::Colorize;
use miette::Result;
use std::path::PathBuf;

use super::{OutputFormat, RsrAction};
use crate::rsr::compliance::{ComplianceChecker, ComplianceLevel};
use crate::rsr::requirements::{RsrRequirementClass, RsrRequirementRegistry};
use crate::rsr::schemas::RsrSchemaRegistry;

/// Run the RSR command
pub async fn run(action: RsrAction, verbose: bool) -> Result<()> {
    match action {
        RsrAction::Check { requirement, format } => {
            run_check(requirement, format, verbose).await
        }
        RsrAction::Requirements { tag, id } => {
            run_requirements(tag, id, verbose).await
        }
        RsrAction::Schemas { tag } => {
            run_schemas(tag, verbose).await
        }
        RsrAction::Schema { id, output } => {
            run_schema(id, output, verbose).await
        }
    }
}

async fn run_check(
    requirements: Vec<String>,
    format: OutputFormat,
    verbose: bool,
) -> Result<()> {
    let checker = ComplianceChecker::new();
    let working_dir = std::env::current_dir()
        .map_err(|e| miette::miette!("Failed to get current directory: {}", e))?;

    if requirements.is_empty() {
        // Check all requirements
        let report = checker.check(&working_dir)?;

        match format {
            OutputFormat::Text => print_compliance_report(&report, verbose),
            OutputFormat::Json => print_compliance_json(&report)?,
        }

        if report.level == ComplianceLevel::NonCompliant {
            return Err(miette::miette!("Compliance check failed"));
        }
    } else {
        // Check specific requirements
        let req_refs: Vec<&str> = requirements.iter().map(|s| s.as_str()).collect();
        let results = checker.check_requirements(&req_refs, &working_dir)?;

        match format {
            OutputFormat::Text => print_requirement_results(&results, verbose),
            OutputFormat::Json => print_requirement_results_json(&results)?,
        }

        if results.iter().any(|r| !r.met) {
            return Err(miette::miette!("Some requirements not met"));
        }
    }

    Ok(())
}

fn print_compliance_report(
    report: &crate::rsr::compliance::ComplianceReport,
    verbose: bool,
) {
    println!();
    println!("{}", "RSR Compliance Report".bold());
    println!("{}", "═".repeat(50));
    println!();

    // Overall level
    let level_color = match report.level {
        ComplianceLevel::Excellent => "green",
        ComplianceLevel::Good => "blue",
        ComplianceLevel::Basic => "yellow",
        ComplianceLevel::NonCompliant => "red",
    };

    println!(
        "Level: {} {}",
        report.level.emoji(),
        report.level.description().color(level_color)
    );
    println!("Score: {:.0}%", report.score * 100.0);
    println!();

    // Stats
    println!("{}:", "Summary".bold());
    println!(
        "  Total:       {}/{} passed",
        report.stats.passed, report.stats.total
    );
    println!(
        "  Mandatory:   {}/{}",
        report.stats.mandatory_passed, report.stats.mandatory_total
    );
    println!(
        "  Preferential: {}/{}",
        report.stats.preferential_passed, report.stats.preferential_total
    );
    println!(
        "  Advisory:    {}/{}",
        report.stats.advisory_passed, report.stats.advisory_total
    );
    println!();

    // Individual requirements
    println!("{}:", "Requirements".bold());
    for result in &report.requirements {
        let icon = if result.met {
            "✓".green()
        } else {
            "✗".red()
        };
        println!("  {} {}", icon, result.requirement_id);

        if verbose && !result.met {
            if let Some(ref rem) = result.remediation {
                for line in rem.lines() {
                    println!("      {}", line.dimmed());
                }
            }
        }
    }

    // Suggestions for failed requirements
    let failed: Vec<_> = report.requirements.iter().filter(|r| !r.met).collect();
    if !failed.is_empty() {
        println!();
        println!("{}:", "Suggestions".bold());
        for result in failed.iter().take(3) {
            println!("  {} {}", "→".blue(), result.requirement_id);
            if let Some(ref rem) = result.remediation {
                let first_line = rem.lines().next().unwrap_or("");
                println!("    {}", first_line);
            }
        }
        if failed.len() > 3 {
            println!("  ... and {} more", failed.len() - 3);
        }
    }

    println!();
}

fn print_compliance_json(
    report: &crate::rsr::compliance::ComplianceReport,
) -> Result<()> {
    let json = serde_json::json!({
        "level": format!("{:?}", report.level),
        "score": report.score,
        "stats": {
            "total": report.stats.total,
            "passed": report.stats.passed,
            "failed": report.stats.failed,
            "mandatory": {
                "total": report.stats.mandatory_total,
                "passed": report.stats.mandatory_passed,
            },
            "preferential": {
                "total": report.stats.preferential_total,
                "passed": report.stats.preferential_passed,
            },
            "advisory": {
                "total": report.stats.advisory_total,
                "passed": report.stats.advisory_passed,
            },
        },
        "requirements": report.requirements.iter().map(|r| {
            serde_json::json!({
                "id": r.requirement_id,
                "met": r.met,
                "remediation": r.remediation,
            })
        }).collect::<Vec<_>>(),
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&json)
            .map_err(|e| miette::miette!("Failed to serialize JSON: {}", e))?
    );

    Ok(())
}

fn print_requirement_results(
    results: &[crate::rsr::compliance::RequirementResult],
    verbose: bool,
) {
    println!();
    println!("{}", "Requirement Check Results".bold());
    println!("{}", "═".repeat(50));
    println!();

    for result in results {
        let icon = if result.met {
            "✓".green()
        } else {
            "✗".red()
        };
        println!("{} {}", icon, result.requirement_id.bold());

        if verbose {
            for detail in &result.details {
                let detail_icon = if detail.passed { "✓" } else { "✗" };
                println!(
                    "    {} {}",
                    if detail.passed {
                        detail_icon.green()
                    } else {
                        detail_icon.red()
                    },
                    detail.check
                );
                if let Some(ref info) = detail.info {
                    println!("      {}", info.dimmed());
                }
            }
        }

        if !result.met {
            if let Some(ref rem) = result.remediation {
                println!("  {}:", "Remediation".yellow());
                for line in rem.lines() {
                    println!("    {}", line);
                }
            }
        }

        println!();
    }
}

fn print_requirement_results_json(
    results: &[crate::rsr::compliance::RequirementResult],
) -> Result<()> {
    let json: Vec<_> = results
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.requirement_id,
                "met": r.met,
                "details": r.details.iter().map(|d| {
                    serde_json::json!({
                        "check": d.check,
                        "passed": d.passed,
                        "info": d.info,
                    })
                }).collect::<Vec<_>>(),
                "remediation": r.remediation,
            })
        })
        .collect();

    println!(
        "{}",
        serde_json::to_string_pretty(&json)
            .map_err(|e| miette::miette!("Failed to serialize JSON: {}", e))?
    );

    Ok(())
}

async fn run_requirements(
    tag: Option<String>,
    id: Option<String>,
    _verbose: bool,
) -> Result<()> {
    let registry = RsrRequirementRegistry::new();

    println!();
    println!("{}", "RSR Requirements".bold());
    println!("{}", "═".repeat(50));
    println!();

    if let Some(ref req_id) = id {
        // Show specific requirement
        if let Some(req) = registry.get(req_id) {
            print_requirement(req);
        } else {
            return Err(miette::miette!("Requirement not found: {}", req_id));
        }
    } else if let Some(ref tag_filter) = tag {
        // Filter by tag
        let reqs = registry.by_tag(tag_filter);
        if reqs.is_empty() {
            println!("No requirements found with tag: {}", tag_filter);
        } else {
            for req in reqs {
                print_requirement_summary(req);
            }
        }
    } else {
        // Show all
        for req in registry.all() {
            print_requirement_summary(req);
        }
    }

    println!();
    Ok(())
}

fn print_requirement_summary(req: &crate::rsr::requirements::RsrRequirement) {
    let class_str = match req.class {
        RsrRequirementClass::Mandatory => "mandatory".red(),
        RsrRequirementClass::Preferential => "preferential".yellow(),
        RsrRequirementClass::Advisory => "advisory".dimmed(),
    };

    println!("{} [{}]", req.id.bold(), class_str);
    println!("  {}", req.name);
    if !req.tags.is_empty() {
        println!("  Tags: {}", req.tags.join(", ").dimmed());
    }
    println!();
}

fn print_requirement(req: &crate::rsr::requirements::RsrRequirement) {
    let class_str = match req.class {
        RsrRequirementClass::Mandatory => "mandatory".red(),
        RsrRequirementClass::Preferential => "preferential".yellow(),
        RsrRequirementClass::Advisory => "advisory".dimmed(),
    };

    println!("{} [{}]", req.id.bold(), class_str);
    println!();
    println!("{}:", "Name".bold());
    println!("  {}", req.name);
    println!();
    println!("{}:", "Description".bold());
    println!("  {}", req.description);
    println!();

    if !req.tags.is_empty() {
        println!("{}:", "Tags".bold());
        println!("  {}", req.tags.join(", "));
        println!();
    }

    if !req.related.is_empty() {
        println!("{}:", "Related".bold());
        for rel in &req.related {
            println!("  - {}", rel);
        }
        println!();
    }

    println!("{}:", "Validation".bold());
    if !req.validation.file_exists.is_empty() {
        println!("  Files required: {:?}", req.validation.file_exists);
    }
    if req.validation.conflow_valid {
        println!("  conflow pipeline must be valid");
    }
    println!();

    println!("{}:", "Remediation".bold());
    if req.remediation.auto_fix {
        println!("  Auto-fix available");
    }
    for step in &req.remediation.manual_steps {
        println!("  • {}", step);
    }
    if let Some(ref url) = req.remediation.docs_url {
        println!("  Docs: {}", url.cyan());
    }
}

async fn run_schemas(tag: Option<String>, _verbose: bool) -> Result<()> {
    let registry = RsrSchemaRegistry::new();

    println!();
    println!("{}", "RSR Schemas".bold());
    println!("{}", "═".repeat(50));
    println!();

    let schemas: Vec<_> = if let Some(ref tag_filter) = tag {
        registry.by_tag(tag_filter)
    } else {
        registry.list().collect()
    };

    if schemas.is_empty() {
        println!("No schemas found");
    } else {
        for schema in schemas {
            println!("{}", schema.id.bold());
            println!("  {} (v{})", schema.name, schema.version);
            println!("  {}", schema.description.dimmed());
            if !schema.tags.is_empty() {
                println!("  Tags: {}", schema.tags.join(", "));
            }
            println!();
        }
    }

    Ok(())
}

async fn run_schema(
    id: String,
    output: Option<PathBuf>,
    _verbose: bool,
) -> Result<()> {
    let registry = RsrSchemaRegistry::new();

    let content = registry.get_content(&id)?;

    if let Some(path) = output {
        std::fs::write(&path, &content)
            .map_err(|e| miette::miette!("Failed to write schema: {}", e))?;
        println!("Schema written to: {}", path.display());
    } else {
        println!("{}", content);
    }

    Ok(())
}
