// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! Progress spinner utilities
//!
//! Provides progress indicators for long-running operations.

use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Create a spinner for indeterminate progress
pub fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.blue} {msg}")
            .expect("Invalid spinner template"),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

/// Create a progress bar for determinate progress
pub fn create_progress_bar(total: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .expect("Invalid progress bar template")
            .progress_chars("█▓░"),
    );
    pb.set_message(message.to_string());
    pb
}

/// A multi-stage progress indicator
pub struct StageProgress {
    stages: Vec<String>,
    current: usize,
}

impl StageProgress {
    pub fn new(stages: Vec<String>) -> Self {
        Self { stages, current: 0 }
    }

    pub fn start(&mut self) {
        if let Some(stage) = self.stages.get(self.current) {
            println!("  → {}...", stage);
        }
    }

    pub fn complete(&mut self) {
        use colored::Colorize;

        if let Some(stage) = self.stages.get(self.current) {
            // Move cursor up and overwrite
            println!("\x1b[1A\x1b[2K  {} {}", "✓".green(), stage);
        }
        self.current += 1;
    }

    pub fn fail(&mut self, error: &str) {
        use colored::Colorize;

        if let Some(stage) = self.stages.get(self.current) {
            println!("\x1b[1A\x1b[2K  {} {} - {}", "✗".red(), stage, error.dimmed());
        }
    }

    pub fn skip(&mut self) {
        use colored::Colorize;

        if let Some(stage) = self.stages.get(self.current) {
            println!("\x1b[1A\x1b[2K  {} {} (skipped)", "○".dimmed(), stage.dimmed());
        }
        self.current += 1;
    }

    pub fn is_complete(&self) -> bool {
        self.current >= self.stages.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_progress() {
        let mut progress = StageProgress::new(vec![
            "Stage 1".to_string(),
            "Stage 2".to_string(),
        ]);

        assert!(!progress.is_complete());
        progress.complete();
        assert!(!progress.is_complete());
        progress.complete();
        assert!(progress.is_complete());
    }
}
