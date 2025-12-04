// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! Complexity analysis for configuration files

use super::ConfigFormat;

/// Complexity analysis result
#[derive(Debug, Clone)]
pub struct Complexity {
    /// Contains conditional logic (if/else, match, etc.)
    pub has_logic: bool,
    /// Contains function definitions
    pub has_functions: bool,
    /// Contains constraint expressions
    pub has_constraints: bool,
    /// Contains validation patterns
    pub has_validation: bool,
    /// Contains generation patterns (loops, comprehensions)
    pub has_generation: bool,
    /// Line count
    pub line_count: usize,
    /// Maximum nesting depth
    pub nesting_depth: usize,
    /// Contains repeated similar structures
    pub has_repetition: bool,
}

/// Analyze the complexity of configuration content
pub fn analyze_complexity(content: &str, format: ConfigFormat) -> Complexity {
    let mut complexity = Complexity {
        has_logic: false,
        has_functions: false,
        has_constraints: false,
        has_validation: false,
        has_generation: false,
        line_count: content.lines().count(),
        nesting_depth: 0,
        has_repetition: false,
    };

    // Check for logic patterns
    complexity.has_logic = check_logic_patterns(content);

    // Check for function patterns
    complexity.has_functions = check_function_patterns(content);

    // Check for constraint patterns
    complexity.has_constraints = check_constraint_patterns(content);

    // Check for generation patterns
    complexity.has_generation = check_generation_patterns(content);

    // Calculate nesting depth
    complexity.nesting_depth = calculate_nesting_depth(content);

    // Check for repetition
    complexity.has_repetition = check_repetition(content);

    // Infer validation if constraints are present
    complexity.has_validation = complexity.has_constraints;

    complexity
}

fn check_logic_patterns(content: &str) -> bool {
    // Common conditional patterns across formats
    let patterns = [
        "if ", "else ", "then ", " ? ", " : ", // Ternary and conditionals
        "match ", "case ", "when ",            // Pattern matching
        "&&", "||", " and ", " or ",           // Logical operators
    ];

    patterns.iter().any(|p| content.contains(p))
}

fn check_function_patterns(content: &str) -> bool {
    let patterns = [
        "fun ",      // Nickel
        "func ",     // Go/CUE style
        "function ", // JavaScript style
        "def ",      // Python style
        "fn ",       // Rust style
        "=>",        // Lambda/arrow functions
        "->",        // Function type annotations
        "lambda ",   // Lambda keyword
    ];

    patterns.iter().any(|p| content.contains(p))
}

fn check_constraint_patterns(content: &str) -> bool {
    let patterns = [
        ">=", "<=", ">", "<", // Comparison operators
        "& ", "| ",           // CUE unification/disjunction
        "=~",                 // Regex matching
        "!~",                 // Negative regex
        "min:", "max:",       // JSON Schema style
        "minLength", "maxLength",
        "pattern:",
        "| *",  // CUE default
        "_|_",  // CUE bottom
    ];

    patterns.iter().any(|p| content.contains(p))
}

fn check_generation_patterns(content: &str) -> bool {
    let patterns = [
        "for ", "foreach ",           // Loop patterns
        "map(", "filter(", "fold(",   // Functional patterns
        "Array.from", "Array.map",    // Array generation
        "std.range", "std.map",       // Nickel stdlib
        "[for ", "{ for ",            // CUE comprehensions
        "...",                        // Spread operators
    ];

    patterns.iter().any(|p| content.contains(p))
}

fn calculate_nesting_depth(content: &str) -> usize {
    let mut max_depth: usize = 0;
    let mut current_depth: usize = 0;

    for ch in content.chars() {
        match ch {
            '{' | '[' | '(' => {
                current_depth += 1;
                max_depth = max_depth.max(current_depth);
            }
            '}' | ']' | ')' => {
                current_depth = current_depth.saturating_sub(1);
            }
            _ => {}
        }
    }

    max_depth
}

fn check_repetition(content: &str) -> bool {
    // Simple heuristic: check if there are multiple similar blocks
    let lines: Vec<&str> = content.lines().collect();

    if lines.len() < 10 {
        return false;
    }

    // Look for repeated patterns (simplified)
    let mut pattern_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();

    for line in &lines {
        let trimmed = line.trim();
        if trimmed.len() > 5 {
            // Only count meaningful lines
            *pattern_counts.entry(trimmed).or_insert(0) += 1;
        }
    }

    // If any pattern appears more than 3 times, consider it repetitive
    pattern_counts.values().any(|&count| count > 3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_logic() {
        let content = "if x > 10 then 'large' else 'small'";
        let complexity = analyze_complexity(content, ConfigFormat::Nickel);
        assert!(complexity.has_logic);
    }

    #[test]
    fn test_detect_functions() {
        let content = "let double = fun x => x * 2 in { value = double 5 }";
        let complexity = analyze_complexity(content, ConfigFormat::Nickel);
        assert!(complexity.has_functions);
    }

    #[test]
    fn test_detect_constraints() {
        let content = "#Config: { port: int & >=1 & <=65535 }";
        let complexity = analyze_complexity(content, ConfigFormat::Cue);
        assert!(complexity.has_constraints);
    }

    #[test]
    fn test_nesting_depth() {
        let content = "{ a: { b: { c: { d: 1 } } } }";
        let complexity = analyze_complexity(content, ConfigFormat::Json);
        assert_eq!(complexity.nesting_depth, 4);
    }

    #[test]
    fn test_simple_config() {
        let content = r#"{"name": "test", "value": 42}"#;
        let complexity = analyze_complexity(content, ConfigFormat::Json);
        assert!(!complexity.has_logic);
        assert!(!complexity.has_functions);
        assert!(!complexity.has_constraints);
    }
}
