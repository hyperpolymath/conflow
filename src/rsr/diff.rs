// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! Compliance diff reports
//!
//! Track changes between compliance runs and generate diff reports.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::compliance::{ComplianceLevel, ComplianceReport, RequirementResult};
use crate::ConflowError;

/// Diff between two compliance reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceDiff {
    /// Previous report timestamp
    pub previous_timestamp: Option<String>,

    /// Current report timestamp
    pub current_timestamp: String,

    /// Level change
    pub level_change: LevelChange,

    /// Score change
    pub score_change: ScoreChange,

    /// Requirements that changed status
    pub requirement_changes: Vec<RequirementChange>,

    /// Summary statistics
    pub summary: DiffSummary,
}

/// Level change between reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelChange {
    pub previous: Option<ComplianceLevel>,
    pub current: ComplianceLevel,
    pub direction: ChangeDirection,
}

/// Score change between reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreChange {
    pub previous: Option<f64>,
    pub current: f64,
    pub delta: f64,
    pub percentage_change: f64,
}

/// Direction of change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeDirection {
    Improved,
    Degraded,
    Unchanged,
    New,
}

/// Change in a specific requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementChange {
    pub requirement_id: String,
    pub previous_met: Option<bool>,
    pub current_met: bool,
    pub change_type: RequirementChangeType,
}

/// Type of requirement change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequirementChangeType {
    /// Was failing, now passing
    Fixed,
    /// Was passing, now failing
    Regressed,
    /// New requirement added
    New,
    /// Requirement removed
    Removed,
    /// No change
    Unchanged,
}

/// Summary of diff
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiffSummary {
    pub total_requirements: usize,
    pub fixed: usize,
    pub regressed: usize,
    pub new_passing: usize,
    pub new_failing: usize,
    pub unchanged_passing: usize,
    pub unchanged_failing: usize,
}

/// Compliance history storage
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComplianceHistory {
    /// History entries, newest first
    pub entries: Vec<HistoryEntry>,
}

/// A single history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub timestamp: String,
    pub level: ComplianceLevel,
    pub score: f64,
    pub requirements: HashMap<String, bool>,
    pub git_commit: Option<String>,
}

impl ComplianceHistory {
    /// Create new empty history
    pub fn new() -> Self {
        Self::default()
    }

    /// Load history from file
    pub fn load(path: &Path) -> Result<Self, ConflowError> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let content = std::fs::read_to_string(path).map_err(|e| ConflowError::Io {
            message: e.to_string(),
        })?;

        serde_json::from_str(&content).map_err(|e| ConflowError::Json {
            message: e.to_string(),
        })
    }

    /// Save history to file
    pub fn save(&self, path: &Path) -> Result<(), ConflowError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ConflowError::Io {
                message: e.to_string(),
            })?;
        }

        let content = serde_json::to_string_pretty(self).map_err(|e| ConflowError::Json {
            message: e.to_string(),
        })?;

        std::fs::write(path, content).map_err(|e| ConflowError::Io {
            message: e.to_string(),
        })?;

        Ok(())
    }

    /// Add a new entry from a compliance report
    pub fn add_entry(&mut self, report: &ComplianceReport, git_commit: Option<String>) {
        let requirements: HashMap<String, bool> = report
            .requirements
            .iter()
            .map(|r| (r.requirement_id.clone(), r.met))
            .collect();

        let entry = HistoryEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            level: report.level,
            score: report.score,
            requirements,
            git_commit,
        };

        self.entries.insert(0, entry);

        // Keep only last 100 entries
        if self.entries.len() > 100 {
            self.entries.truncate(100);
        }
    }

    /// Get the most recent entry
    pub fn latest(&self) -> Option<&HistoryEntry> {
        self.entries.first()
    }

    /// Get the previous entry (second most recent)
    pub fn previous(&self) -> Option<&HistoryEntry> {
        self.entries.get(1)
    }

    /// Generate diff between latest and previous
    pub fn diff_latest(&self) -> Option<ComplianceDiff> {
        let current = self.latest()?;
        let previous = self.previous();

        Some(Self::diff_entries(previous, current))
    }

    /// Generate diff between any two entries
    pub fn diff_entries(previous: Option<&HistoryEntry>, current: &HistoryEntry) -> ComplianceDiff {
        let level_change = LevelChange {
            previous: previous.map(|p| p.level),
            current: current.level,
            direction: match previous {
                None => ChangeDirection::New,
                Some(p) if current.level > p.level => ChangeDirection::Improved,
                Some(p) if current.level < p.level => ChangeDirection::Degraded,
                _ => ChangeDirection::Unchanged,
            },
        };

        let score_change = ScoreChange {
            previous: previous.map(|p| p.score),
            current: current.score,
            delta: current.score - previous.map(|p| p.score).unwrap_or(0.0),
            percentage_change: previous
                .map(|p| {
                    if p.score > 0.0 {
                        ((current.score - p.score) / p.score) * 100.0
                    } else {
                        0.0
                    }
                })
                .unwrap_or(0.0),
        };

        let mut requirement_changes = Vec::new();
        let mut summary = DiffSummary::default();

        // Collect all requirement IDs
        let mut all_ids: Vec<String> = current.requirements.keys().cloned().collect();
        if let Some(prev) = previous {
            for id in prev.requirements.keys() {
                if !all_ids.contains(id) {
                    all_ids.push(id.clone());
                }
            }
        }

        for id in all_ids {
            let current_met = current.requirements.get(&id).copied();
            let previous_met = previous.and_then(|p| p.requirements.get(&id).copied());

            let change_type = match (previous_met, current_met) {
                (None, Some(true)) => RequirementChangeType::New,
                (None, Some(false)) => RequirementChangeType::New,
                (None, None) => continue,
                (Some(_), None) => RequirementChangeType::Removed,
                (Some(false), Some(true)) => RequirementChangeType::Fixed,
                (Some(true), Some(false)) => RequirementChangeType::Regressed,
                (Some(true), Some(true)) => RequirementChangeType::Unchanged,
                (Some(false), Some(false)) => RequirementChangeType::Unchanged,
            };

            // Update summary
            summary.total_requirements += 1;
            match change_type {
                RequirementChangeType::Fixed => summary.fixed += 1,
                RequirementChangeType::Regressed => summary.regressed += 1,
                RequirementChangeType::New => {
                    if current_met.unwrap_or(false) {
                        summary.new_passing += 1;
                    } else {
                        summary.new_failing += 1;
                    }
                }
                RequirementChangeType::Unchanged => {
                    if current_met.unwrap_or(false) {
                        summary.unchanged_passing += 1;
                    } else {
                        summary.unchanged_failing += 1;
                    }
                }
                RequirementChangeType::Removed => {}
            }

            requirement_changes.push(RequirementChange {
                requirement_id: id,
                previous_met,
                current_met: current_met.unwrap_or(false),
                change_type,
            });
        }

        ComplianceDiff {
            previous_timestamp: previous.map(|p| p.timestamp.clone()),
            current_timestamp: current.timestamp.clone(),
            level_change,
            score_change,
            requirement_changes,
            summary,
        }
    }

    /// Get trend over time
    pub fn trend(&self, count: usize) -> Vec<(String, f64)> {
        self.entries
            .iter()
            .take(count)
            .map(|e| (e.timestamp.clone(), e.score))
            .collect()
    }
}

/// Diff reporter for CLI output
pub struct DiffReporter;

impl DiffReporter {
    /// Format diff for CLI output
    pub fn format_text(diff: &ComplianceDiff) -> String {
        let mut output = String::new();

        output.push_str("Compliance Diff Report\n");
        output.push_str(&"═".repeat(50));
        output.push('\n');

        // Level change
        let level_emoji = match diff.level_change.direction {
            ChangeDirection::Improved => "📈",
            ChangeDirection::Degraded => "📉",
            ChangeDirection::Unchanged => "➡️",
            ChangeDirection::New => "🆕",
        };

        output.push_str(&format!(
            "\nLevel: {} {:?} → {:?}\n",
            level_emoji,
            diff.level_change.previous.unwrap_or(ComplianceLevel::NonCompliant),
            diff.level_change.current
        ));

        // Score change
        let score_sign = if diff.score_change.delta >= 0.0 { "+" } else { "" };
        output.push_str(&format!(
            "Score: {:.0}% ({}{:.1}%)\n",
            diff.score_change.current * 100.0,
            score_sign,
            diff.score_change.delta * 100.0
        ));

        // Summary
        output.push_str("\nSummary:\n");
        if diff.summary.fixed > 0 {
            output.push_str(&format!("  ✅ {} fixed\n", diff.summary.fixed));
        }
        if diff.summary.regressed > 0 {
            output.push_str(&format!("  ❌ {} regressed\n", diff.summary.regressed));
        }
        if diff.summary.new_passing > 0 {
            output.push_str(&format!("  🆕 {} new (passing)\n", diff.summary.new_passing));
        }
        if diff.summary.new_failing > 0 {
            output.push_str(&format!("  🆕 {} new (failing)\n", diff.summary.new_failing));
        }

        // Requirement details
        let changes: Vec<_> = diff
            .requirement_changes
            .iter()
            .filter(|c| c.change_type != RequirementChangeType::Unchanged)
            .collect();

        if !changes.is_empty() {
            output.push_str("\nChanges:\n");
            for change in changes {
                let icon = match change.change_type {
                    RequirementChangeType::Fixed => "✅",
                    RequirementChangeType::Regressed => "❌",
                    RequirementChangeType::New => "🆕",
                    RequirementChangeType::Removed => "🗑️",
                    RequirementChangeType::Unchanged => "➡️",
                };
                output.push_str(&format!("  {} {}\n", icon, change.requirement_id));
            }
        }

        output
    }

    /// Format diff as JSON
    pub fn format_json(diff: &ComplianceDiff) -> Result<String, ConflowError> {
        serde_json::to_string_pretty(diff).map_err(|e| ConflowError::Json {
            message: e.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rsr::compliance::ComplianceStats;

    fn sample_report(level: ComplianceLevel, score: f64, requirements: Vec<(&str, bool)>) -> ComplianceReport {
        ComplianceReport {
            level,
            score,
            requirements: requirements
                .into_iter()
                .map(|(id, met)| RequirementResult {
                    requirement_id: id.to_string(),
                    met,
                    details: vec![],
                    remediation: None,
                })
                .collect(),
            stats: ComplianceStats::default(),
        }
    }

    #[test]
    fn test_history_add_and_diff() {
        let mut history = ComplianceHistory::new();

        // Add first report
        let report1 = sample_report(
            ComplianceLevel::Basic,
            0.6,
            vec![("RSR-001", false), ("RSR-002", true)],
        );
        history.add_entry(&report1, None);

        // Add second report (improved)
        let report2 = sample_report(
            ComplianceLevel::Good,
            0.8,
            vec![("RSR-001", true), ("RSR-002", true)],
        );
        history.add_entry(&report2, None);

        let diff = history.diff_latest().unwrap();

        assert_eq!(diff.level_change.direction, ChangeDirection::Improved);
        assert!(diff.score_change.delta > 0.0);
        assert_eq!(diff.summary.fixed, 1);
    }

    #[test]
    fn test_diff_regression() {
        let mut history = ComplianceHistory::new();

        let report1 = sample_report(
            ComplianceLevel::Good,
            0.8,
            vec![("RSR-001", true), ("RSR-002", true)],
        );
        history.add_entry(&report1, None);

        let report2 = sample_report(
            ComplianceLevel::Basic,
            0.5,
            vec![("RSR-001", false), ("RSR-002", true)],
        );
        history.add_entry(&report2, None);

        let diff = history.diff_latest().unwrap();

        assert_eq!(diff.level_change.direction, ChangeDirection::Degraded);
        assert!(diff.score_change.delta < 0.0);
        assert_eq!(diff.summary.regressed, 1);
    }

    #[test]
    fn test_format_text() {
        let mut history = ComplianceHistory::new();

        let report1 = sample_report(ComplianceLevel::Basic, 0.5, vec![("RSR-001", false)]);
        history.add_entry(&report1, None);

        let report2 = sample_report(ComplianceLevel::Good, 0.8, vec![("RSR-001", true)]);
        history.add_entry(&report2, None);

        let diff = history.diff_latest().unwrap();
        let text = DiffReporter::format_text(&diff);

        assert!(text.contains("Compliance Diff Report"));
        assert!(text.contains("fixed"));
    }
}
