// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! Tool recommendation engine
//!
//! Recommends the appropriate tool (CUE or Nickel) based on complexity analysis.

use super::Complexity;

/// Recommended tool
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecommendedTool {
    Cue,
    Nickel,
}

/// Tool recommendation with rationale
#[derive(Debug, Clone)]
pub struct ToolRecommendation {
    /// Primary recommended tool
    pub primary: RecommendedTool,
    /// Reasons for the recommendation
    pub rationale: Vec<String>,
    /// Alternative options
    pub alternatives: Vec<Alternative>,
    /// Suggested combined approach (if applicable)
    pub combined_approach: Option<String>,
}

/// An alternative tool option
#[derive(Debug, Clone)]
pub struct Alternative {
    pub tool: RecommendedTool,
    pub reason: String,
}

/// Generate a tool recommendation based on complexity analysis
pub fn recommend_tool(complexity: &Complexity) -> ToolRecommendation {
    // Decision tree for tool selection
    //
    // Nickel is preferred when:
    // - Complex logic (if/else, loops)
    // - Functions needed
    // - Configuration generation
    // - High repetition (DRY with functions)
    //
    // CUE is preferred when:
    // - Constraint validation
    // - Schema definition
    // - Simple transformations
    // - Unification/merging needed

    // Count factors favoring each tool
    let mut nickel_score = 0;
    let mut cue_score = 0;

    let mut nickel_reasons = Vec::new();
    let mut cue_reasons = Vec::new();

    // Logic patterns strongly favor Nickel
    if complexity.has_logic {
        nickel_score += 3;
        nickel_reasons.push("Complex logic detected (conditionals, branching)".to_string());
    }

    // Functions strongly favor Nickel
    if complexity.has_functions {
        nickel_score += 3;
        nickel_reasons.push("Function definitions detected".to_string());
    }

    // Generation patterns favor Nickel
    if complexity.has_generation {
        nickel_score += 2;
        nickel_reasons.push("Configuration generation patterns detected".to_string());
    }

    // Repetition suggests Nickel for DRY
    if complexity.has_repetition {
        nickel_score += 1;
        nickel_reasons.push("Repetitive patterns could benefit from abstraction".to_string());
    }

    // Constraints favor CUE
    if complexity.has_constraints {
        cue_score += 3;
        cue_reasons.push("Constraint validation patterns detected".to_string());
    }

    // Validation patterns favor CUE
    if complexity.has_validation {
        cue_score += 2;
        cue_reasons.push("Schema validation requirements detected".to_string());
    }

    // Deep nesting slightly favors CUE (unification handles it well)
    if complexity.nesting_depth > 3 {
        cue_score += 1;
        cue_reasons.push("Deep nesting works well with CUE unification".to_string());
    }

    // Simple configs slightly favor CUE
    if !complexity.has_logic && !complexity.has_functions && complexity.line_count < 50 {
        cue_score += 1;
        cue_reasons.push("Simple configuration structure".to_string());
    }

    // Make recommendation
    let (primary, mut rationale) = if nickel_score > cue_score {
        (RecommendedTool::Nickel, nickel_reasons)
    } else if cue_score > nickel_score {
        (RecommendedTool::Cue, cue_reasons)
    } else {
        // Tie-breaker: prefer CUE for validation, Nickel for generation
        if complexity.has_constraints {
            (RecommendedTool::Cue, cue_reasons)
        } else {
            (RecommendedTool::Nickel, nickel_reasons)
        }
    };

    // Add default reason if none
    if rationale.is_empty() {
        rationale.push(match primary {
            RecommendedTool::Cue => "CUE provides good validation defaults".to_string(),
            RecommendedTool::Nickel => "Nickel provides flexible configuration".to_string(),
        });
    }

    // Build alternatives
    let mut alternatives = Vec::new();

    match primary {
        RecommendedTool::Cue => {
            if complexity.has_repetition || complexity.line_count > 100 {
                alternatives.push(Alternative {
                    tool: RecommendedTool::Nickel,
                    reason: "Consider Nickel if you need more abstraction".to_string(),
                });
            }
        }
        RecommendedTool::Nickel => {
            if complexity.has_constraints {
                alternatives.push(Alternative {
                    tool: RecommendedTool::Cue,
                    reason: "CUE provides stronger constraint validation".to_string(),
                });
            }
        }
    }

    // Suggest combined approach if both have strong signals
    let combined_approach = if nickel_score >= 2 && cue_score >= 2 {
        Some("Use Nickel to generate configurations, then CUE to validate them".to_string())
    } else {
        None
    };

    ToolRecommendation {
        primary,
        rationale,
        alternatives,
        combined_approach,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recommend_cue_for_constraints() {
        let complexity = Complexity {
            has_logic: false,
            has_functions: false,
            has_constraints: true,
            has_validation: true,
            has_generation: false,
            line_count: 50,
            nesting_depth: 2,
            has_repetition: false,
        };

        let rec = recommend_tool(&complexity);
        assert_eq!(rec.primary, RecommendedTool::Cue);
    }

    #[test]
    fn test_recommend_nickel_for_logic() {
        let complexity = Complexity {
            has_logic: true,
            has_functions: true,
            has_constraints: false,
            has_validation: false,
            has_generation: true,
            line_count: 100,
            nesting_depth: 3,
            has_repetition: true,
        };

        let rec = recommend_tool(&complexity);
        assert_eq!(rec.primary, RecommendedTool::Nickel);
    }

    #[test]
    fn test_combined_approach_suggested() {
        let complexity = Complexity {
            has_logic: true,
            has_functions: true,
            has_constraints: true,
            has_validation: true,
            has_generation: false,
            line_count: 100,
            nesting_depth: 3,
            has_repetition: false,
        };

        let rec = recommend_tool(&complexity);
        assert!(rec.combined_approach.is_some());
    }
}
