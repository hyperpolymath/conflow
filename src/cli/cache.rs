// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! Cache command - manage the cache

use colored::Colorize;
use miette::Result;
use std::io::{self, Write};

use super::CacheAction;
use crate::cache::{Cache, FilesystemCache};

/// Run the cache command
pub async fn run(action: CacheAction, _verbose: bool) -> Result<()> {
    let working_dir = std::env::current_dir().map_err(|e| {
        miette::miette!("Failed to get current directory: {}", e)
    })?;

    let cache_dir = working_dir.join(".conflow").join("cache");
    let cache = FilesystemCache::new(cache_dir.clone(), working_dir)?;

    match action {
        CacheAction::Stats => {
            let stats = cache.stats().await?;

            println!("{}", "Cache Statistics".bold());
            println!("{}", "═".repeat(40));
            println!("  Location: {}", cache_dir.display());
            println!("  Entries:  {}", stats.entries);
            println!("  Size:     {}", stats.formatted_size());

            if let Some(oldest) = stats.oldest_entry {
                if let Ok(duration) = oldest.elapsed() {
                    println!("  Oldest:   {} ago", format_duration(duration));
                }
            }

            if let Some(newest) = stats.newest_entry {
                if let Ok(duration) = newest.elapsed() {
                    println!("  Newest:   {} ago", format_duration(duration));
                }
            }

            Ok(())
        }

        CacheAction::Clear { yes } => {
            let stats = cache.stats().await?;

            if stats.entries == 0 {
                println!("{}", "Cache is already empty.".dimmed());
                return Ok(());
            }

            if !yes {
                print!(
                    "Clear {} cache entries ({})? [y/N] ",
                    stats.entries,
                    stats.formatted_size()
                );
                io::stdout().flush().ok();

                let mut input = String::new();
                io::stdin().read_line(&mut input).ok();

                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("{}", "Cancelled.".dimmed());
                    return Ok(());
                }
            }

            cache.clear().await?;
            println!("{}", "Cache cleared.".green());

            Ok(())
        }

        CacheAction::List => {
            // For now, just show stats since we don't expose entry listing in the trait
            let stats = cache.stats().await?;

            println!("{}", "Cached Entries".bold());
            println!("{}", "═".repeat(40));

            if stats.entries == 0 {
                println!("{}", "  No cached entries.".dimmed());
            } else {
                println!("  {} entries ({})", stats.entries, stats.formatted_size());
                println!();
                println!(
                    "{}",
                    "  Run 'conflow run --no-cache' to bypass cache.".dimmed()
                );
            }

            Ok(())
        }
    }
}

fn format_duration(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();

    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86400 {
        format!("{}h", secs / 3600)
    } else {
        format!("{}d", secs / 86400)
    }
}
