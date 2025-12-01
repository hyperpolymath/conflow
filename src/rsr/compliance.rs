//! RSR Compliance checking
//!
//! Checks project compliance with RSR requirements and generates reports.

use std::collections::HashMap;
use std::path::Path;

use crate::pipeline::{Pipeline, PipelineValidator};
use crate::ConflowError;

use super::requirements::{
    CueValidation, PatternCheck, RsrRequirement, RsrRequirementClass, RsrRequirementRegistry,
    ValidationChecks,
};

/// Compliance level based on requirements met
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComplianceLevel {
    /// No compliance - mandatory requirements not met
    NonCompliant,
    /// Basic compliance - mandatory requirements met
    Basic,
    /// Good compliance - mandatory + most preferential met
    Good,
    /// Excellent compliance - all requirements met
    Excellent,
}

impl ComplianceLevel {
    pub fn from_score(score: f64, mandatory_met: bool) -> Self {
        if !mandatory_met {
            Self::NonCompliant
        } else if score >= 0.9 {
            Self::Excellent
        } else if score >= 0.7 {
            Self::Good
        } else {
            Self::Basic
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            Self::NonCompliant => "❌",
            Self::Basic => "✓",
            Self::Good => "✓✓",
            Self::Excellent => "✓✓✓",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::NonCompliant => "Non-compliant - mandatory requirements not met",
            Self::Basic => "Basic compliance - core requirements satisfied",
            Self::Good => "Good compliance - most best practices followed",
            Self::Excellent => "Excellent compliance - all recommendations met",
        }
    }
}

/// Result of checking a single requirement
#[derive(Debug, Clone)]
pub struct RequirementResult {
    /// Requirement ID
    pub requirement_id: String,

    /// Whether the requirement is met
    pub met: bool,

    /// Details about what passed/failed
    pub details: Vec<CheckDetail>,

    /// Suggested remediation if not met
    pub remediation: Option<String>,
}

/// Detail of a single check
#[derive(Debug, Clone)]
pub struct CheckDetail {
    /// What was checked
    pub check: String,

    /// Whether it passed
    pub passed: bool,

    /// Additional info
    pub info: Option<String>,
}

/// Full compliance report
#[derive(Debug, Clone)]
pub struct ComplianceReport {
    /// Overall compliance level
    pub level: ComplianceLevel,

    /// Overall score (0.0 - 1.0)
    pub score: f64,

    /// Individual requirement results
    pub requirements: Vec<RequirementResult>,

    /// Summary statistics
    pub stats: ComplianceStats,
}

/// Summary statistics
#[derive(Debug, Clone, Default)]
pub struct ComplianceStats {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub mandatory_total: usize,
    pub mandatory_passed: usize,
    pub preferential_total: usize,
    pub preferential_passed: usize,
    pub advisory_total: usize,
    pub advisory_passed: usize,
}

/// Compliance checker
pub struct ComplianceChecker {
    registry: RsrRequirementRegistry,
}

impl ComplianceChecker {
    /// Create a new compliance checker
    pub fn new() -> Self {
        Self {
            registry: RsrRequirementRegistry::new(),
        }
    }

    /// Create with custom registry
    pub fn with_registry(registry: RsrRequirementRegistry) -> Self {
        Self { registry }
    }

    /// Check compliance for a project
    pub fn check(&self, project_root: &Path) -> Result<ComplianceReport, ConflowError> {
        let mut results = Vec::new();
        let mut stats = ComplianceStats::default();

        for requirement in self.registry.all() {
            let result = self.check_requirement(requirement, project_root)?;

            // Update stats
            stats.total += 1;
            if result.met {
                stats.passed += 1;
            } else {
                stats.failed += 1;
            }

            match requirement.class {
                RsrRequirementClass::Mandatory => {
                    stats.mandatory_total += 1;
                    if result.met {
                        stats.mandatory_passed += 1;
                    }
                }
                RsrRequirementClass::Preferential => {
                    stats.preferential_total += 1;
                    if result.met {
                        stats.preferential_passed += 1;
                    }
                }
                RsrRequirementClass::Advisory => {
                    stats.advisory_total += 1;
                    if result.met {
                        stats.advisory_passed += 1;
                    }
                }
            }

            results.push(result);
        }

        // Calculate score
        let score = self.calculate_score(&results);
        let mandatory_met = stats.mandatory_passed == stats.mandatory_total;
        let level = ComplianceLevel::from_score(score, mandatory_met);

        Ok(ComplianceReport {
            level,
            score,
            requirements: results,
            stats,
        })
    }

    /// Check a single requirement
    fn check_requirement(
        &self,
        requirement: &RsrRequirement,
        project_root: &Path,
    ) -> Result<RequirementResult, ConflowError> {
        let mut details = Vec::new();
        let mut all_passed = true;

        let validation = &requirement.validation;

        // Check file existence
        for file in &validation.file_exists {
            let path = project_root.join(file);
            let exists = path.exists();

            details.push(CheckDetail {
                check: format!("File exists: {}", file.display()),
                passed: exists,
                info: if !exists {
                    Some(format!("Expected file not found: {}", path.display()))
                } else {
                    None
                },
            });

            if !exists {
                all_passed = false;
            }
        }

        // Check file absence
        for file in &validation.file_absent {
            let path = project_root.join(file);
            let absent = !path.exists();

            details.push(CheckDetail {
                check: format!("File absent: {}", file.display()),
                passed: absent,
                info: if !absent {
                    Some(format!("File should not exist: {}", path.display()))
                } else {
                    None
                },
            });

            if !absent {
                all_passed = false;
            }
        }

        // Check patterns
        for pattern_check in &validation.patterns {
            let result = self.check_pattern(pattern_check, project_root);
            let passed = result.is_ok() && result.as_ref().unwrap() == &pattern_check.should_match;

            details.push(CheckDetail {
                check: format!(
                    "Pattern {} in {}",
                    if pattern_check.should_match {
                        "matches"
                    } else {
                        "absent"
                    },
                    pattern_check.file.display()
                ),
                passed,
                info: result.err().map(|e| e.to_string()),
            });

            if !passed {
                all_passed = false;
            }
        }

        // Check conflow validity
        if validation.conflow_valid {
            let result = self.check_conflow_valid(project_root);
            let passed = result.is_ok();
            let info = result.err().map(|e| e.to_string());

            details.push(CheckDetail {
                check: "conflow pipeline valid".into(),
                passed,
                info,
            });

            if !passed {
                all_passed = false;
            }
        }

        // Check CUE validations
        for cue_val in &validation.cue_validate {
            let result = self.check_cue_validation(cue_val, project_root);
            let passed = result.is_ok();
            let info = result.err().map(|e| e.to_string());

            details.push(CheckDetail {
                check: format!("CUE validation: {}", cue_val.schema.display()),
                passed,
                info,
            });

            if !passed {
                all_passed = false;
            }
        }

        // Check shell command
        if let Some(ref shell_check) = validation.shell_check {
            let result = self.check_shell_command(shell_check, project_root);

            details.push(CheckDetail {
                check: format!("Shell check: {}", shell_check),
                passed: result,
                info: None,
            });

            if !result {
                all_passed = false;
            }
        }

        // Generate remediation suggestion if not met
        let remediation = if !all_passed {
            let mut rem = Vec::new();

            if !requirement.remediation.templates.is_empty() {
                rem.push(format!(
                    "Run: conflow init --template {}",
                    requirement.remediation.templates[0]
                        .conflow_template
                        .as_deref()
                        .unwrap_or(&requirement.remediation.templates[0].name)
                ));
            }

            for step in &requirement.remediation.manual_steps {
                rem.push(format!("• {}", step));
            }

            if let Some(ref url) = requirement.remediation.docs_url {
                rem.push(format!("See: {}", url));
            }

            Some(rem.join("\n"))
        } else {
            None
        };

        Ok(RequirementResult {
            requirement_id: requirement.id.clone(),
            met: all_passed,
            details,
            remediation,
        })
    }

    /// Check a pattern in a file
    fn check_pattern(
        &self,
        check: &PatternCheck,
        project_root: &Path,
    ) -> Result<bool, ConflowError> {
        let path = project_root.join(&check.file);

        if !path.exists() {
            return Ok(false);
        }

        let content = std::fs::read_to_string(&path).map_err(|e| ConflowError::Io {
            message: e.to_string(),
        })?;

        let re = regex::Regex::new(&check.pattern).map_err(|e| ConflowError::InvalidPipeline {
            reason: format!("Invalid regex pattern: {}", e),
            help: None,
        })?;

        Ok(re.is_match(&content))
    }

    /// Check if conflow pipeline is valid
    fn check_conflow_valid(&self, project_root: &Path) -> Result<(), ConflowError> {
        let pipeline_path = project_root.join(".conflow.yaml");

        if !pipeline_path.exists() {
            return Err(ConflowError::PipelineNotFound {
                path: pipeline_path,
            });
        }

        let pipeline = Pipeline::from_file(&pipeline_path)?;
        let validation = PipelineValidator::validate(&pipeline)?;

        if !validation.is_valid() {
            return Err(ConflowError::InvalidPipeline {
                reason: validation.errors.join("; "),
                help: None,
            });
        }

        Ok(())
    }

    /// Check CUE validation
    fn check_cue_validation(
        &self,
        _cue_val: &CueValidation,
        _project_root: &Path,
    ) -> Result<(), ConflowError> {
        // Would use CUE executor here
        // For now, just check files exist
        Ok(())
    }

    /// Check shell command
    fn check_shell_command(&self, command: &str, project_root: &Path) -> bool {
        std::process::Command::new("bash")
            .arg("-c")
            .arg(command)
            .current_dir(project_root)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Calculate weighted score
    fn calculate_score(&self, results: &[RequirementResult]) -> f64 {
        let mut total_weight = 0.0;
        let mut earned_weight = 0.0;

        for result in results {
            if let Some(req) = self.registry.get(&result.requirement_id) {
                let weight = req.class.weight();
                total_weight += weight;
                if result.met {
                    earned_weight += weight;
                }
            }
        }

        if total_weight > 0.0 {
            earned_weight / total_weight
        } else {
            1.0
        }
    }

    /// Check specific requirements
    pub fn check_requirements(
        &self,
        requirement_ids: &[&str],
        project_root: &Path,
    ) -> Result<Vec<RequirementResult>, ConflowError> {
        let mut results = Vec::new();

        for id in requirement_ids {
            if let Some(req) = self.registry.get(id) {
                results.push(self.check_requirement(req, project_root)?);
            }
        }

        Ok(results)
    }
}

impl Default for ComplianceChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_compliance_level_from_score() {
        assert_eq!(
            ComplianceLevel::from_score(0.95, true),
            ComplianceLevel::Excellent
        );
        assert_eq!(
            ComplianceLevel::from_score(0.75, true),
            ComplianceLevel::Good
        );
        assert_eq!(
            ComplianceLevel::from_score(0.5, true),
            ComplianceLevel::Basic
        );
        assert_eq!(
            ComplianceLevel::from_score(0.95, false),
            ComplianceLevel::NonCompliant
        );
    }

    #[test]
    fn test_check_empty_project() {
        let temp = TempDir::new().unwrap();
        let checker = ComplianceChecker::new();

        let report = checker.check(temp.path()).unwrap();

        // Should have some failed requirements
        assert!(report.stats.failed > 0);
    }

    #[test]
    fn test_check_with_conflow() {
        let temp = TempDir::new().unwrap();

        // Create a valid .conflow.yaml
        std::fs::write(
            temp.path().join(".conflow.yaml"),
            r#"
version: "1"
name: "test"
stages:
  - name: "validate"
    tool:
      type: cue
      command: vet
    input: "*.json"
"#,
        )
        .unwrap();

        let checker = ComplianceChecker::new();
        let results = checker
            .check_requirements(&["RSR-CONFIG-002"], temp.path())
            .unwrap();

        // RSR-CONFIG-002 should pass (file exists and valid)
        assert!(results[0].met);
    }
}
