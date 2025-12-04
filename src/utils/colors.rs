// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! Terminal color utilities
//!
//! Provides consistent color schemes across the CLI.

use colored::{Color, Colorize};

/// Style for success messages
pub fn success(msg: &str) -> colored::ColoredString {
    msg.green()
}

/// Style for error messages
pub fn error(msg: &str) -> colored::ColoredString {
    msg.red()
}

/// Style for warning messages
pub fn warning(msg: &str) -> colored::ColoredString {
    msg.yellow()
}

/// Style for info messages
pub fn info(msg: &str) -> colored::ColoredString {
    msg.blue()
}

/// Style for dimmed/secondary text
pub fn dimmed(msg: &str) -> colored::ColoredString {
    msg.dimmed()
}

/// Style for emphasized/bold text
pub fn bold(msg: &str) -> colored::ColoredString {
    msg.bold()
}

/// Style for code/commands
pub fn code(msg: &str) -> colored::ColoredString {
    msg.cyan()
}

/// Check if colors should be disabled
pub fn should_use_colors() -> bool {
    // Respect NO_COLOR environment variable
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }

    // Check if stdout is a terminal
    atty_check()
}

fn atty_check() -> bool {
    // Simple check - could be enhanced with atty crate
    std::env::var("TERM").is_ok()
}

/// Print a styled header
pub fn print_header(title: &str) {
    println!("{}", title.bold());
    println!("{}", "═".repeat(title.len().max(40)));
}

/// Print a styled section
pub fn print_section(title: &str) {
    println!();
    println!("{}:", title.bold());
}

/// Print a bullet point
pub fn print_bullet(content: &str) {
    println!("  • {}", content);
}

/// Print a numbered item
pub fn print_numbered(num: usize, content: &str) {
    println!("  {}. {}", num, content);
}

/// Print a success check
pub fn print_success(msg: &str) {
    println!("  {} {}", "✓".green(), msg);
}

/// Print an error cross
pub fn print_error(msg: &str) {
    println!("  {} {}", "✗".red(), msg);
}

/// Print a warning
pub fn print_warning(msg: &str) {
    println!("  {} {}", "⚠".yellow(), msg);
}

/// Print an info item
pub fn print_info(msg: &str) {
    println!("  {} {}", "→".blue(), msg);
}
