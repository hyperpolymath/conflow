// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! Compliance badge generation
//!
//! Generates SVG badges for CI pipelines showing compliance status.

use super::compliance::{ComplianceLevel, ComplianceReport};

/// Badge style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeStyle {
    /// Flat style (shields.io flat)
    Flat,
    /// Flat square style
    FlatSquare,
    /// Plastic style (rounded)
    Plastic,
    /// For the badge style
    ForTheBadge,
}

impl BadgeStyle {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Flat => "flat",
            Self::FlatSquare => "flat-square",
            Self::Plastic => "plastic",
            Self::ForTheBadge => "for-the-badge",
        }
    }
}

impl Default for BadgeStyle {
    fn default() -> Self {
        Self::Flat
    }
}

/// Badge generator
#[derive(Clone)]
pub struct BadgeGenerator {
    style: BadgeStyle,
    label: String,
}

impl BadgeGenerator {
    /// Create a new badge generator
    pub fn new() -> Self {
        Self {
            style: BadgeStyle::default(),
            label: "RSR".into(),
        }
    }

    /// Set badge style
    pub fn style(mut self, style: BadgeStyle) -> Self {
        self.style = style;
        self
    }

    /// Set label text
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Generate SVG badge from compliance report
    pub fn generate(&self, report: &ComplianceReport) -> String {
        let (status, color) = self.level_to_status_color(report.level);
        let score = format!("{:.0}%", report.score * 100.0);

        self.generate_svg(&self.label, &status, color, Some(&score))
    }

    /// Generate a simple level badge
    pub fn generate_level(&self, level: ComplianceLevel) -> String {
        let (status, color) = self.level_to_status_color(level);
        self.generate_svg(&self.label, &status, color, None)
    }

    /// Generate badge with custom status
    pub fn generate_custom(&self, label: &str, status: &str, color: &str) -> String {
        self.generate_svg(label, status, color, None)
    }

    fn level_to_status_color(&self, level: ComplianceLevel) -> (String, &'static str) {
        match level {
            ComplianceLevel::Excellent => ("excellent".into(), "#4c1"),
            ComplianceLevel::Good => ("good".into(), "#97ca00"),
            ComplianceLevel::Basic => ("basic".into(), "#dfb317"),
            ComplianceLevel::NonCompliant => ("non-compliant".into(), "#e05d44"),
        }
    }

    fn generate_svg(&self, label: &str, status: &str, color: &str, score: Option<&str>) -> String {
        let full_status = if let Some(s) = score {
            format!("{} ({})", status, s)
        } else {
            status.to_string()
        };

        match self.style {
            BadgeStyle::Flat => self.flat_badge(label, &full_status, color),
            BadgeStyle::FlatSquare => self.flat_square_badge(label, &full_status, color),
            BadgeStyle::Plastic => self.plastic_badge(label, &full_status, color),
            BadgeStyle::ForTheBadge => self.for_the_badge(label, &full_status, color),
        }
    }

    fn flat_badge(&self, label: &str, status: &str, color: &str) -> String {
        let label_width = self.text_width(label) + 10;
        let status_width = self.text_width(status) + 10;
        let total_width = label_width + status_width;
        let label_x = label_width / 2;
        let status_x = label_width + status_width / 2;

        format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="{total_width}" height="20" role="img" aria-label="{label}: {status}">
  <title>{label}: {status}</title>
  <linearGradient id="s" x2="0" y2="100%">
    <stop offset="0" stop-color="#bbb" stop-opacity=".1"/>
    <stop offset="1" stop-opacity=".1"/>
  </linearGradient>
  <clipPath id="r">
    <rect width="{total_width}" height="20" rx="3" fill="#fff"/>
  </clipPath>
  <g clip-path="url(#r)">
    <rect width="{label_width}" height="20" fill="#555"/>
    <rect x="{label_width}" width="{status_width}" height="20" fill="{color}"/>
    <rect width="{total_width}" height="20" fill="url(#s)"/>
  </g>
  <g fill="#fff" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" text-rendering="geometricPrecision" font-size="110">
    <text aria-hidden="true" x="{label_x}0" y="150" fill="#010101" fill-opacity=".3" transform="scale(.1)">{label}</text>
    <text x="{label_x}0" y="140" transform="scale(.1)" fill="#fff">{label}</text>
    <text aria-hidden="true" x="{status_x}0" y="150" fill="#010101" fill-opacity=".3" transform="scale(.1)">{status}</text>
    <text x="{status_x}0" y="140" transform="scale(.1)" fill="#fff">{status}</text>
  </g>
</svg>"##
        )
    }

    fn flat_square_badge(&self, label: &str, status: &str, color: &str) -> String {
        let label_width = self.text_width(label) + 10;
        let status_width = self.text_width(status) + 10;
        let total_width = label_width + status_width;
        let label_x = label_width / 2;
        let status_x = label_width + status_width / 2;

        format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="{total_width}" height="20" role="img" aria-label="{label}: {status}">
  <title>{label}: {status}</title>
  <g shape-rendering="crispEdges">
    <rect width="{label_width}" height="20" fill="#555"/>
    <rect x="{label_width}" width="{status_width}" height="20" fill="{color}"/>
  </g>
  <g fill="#fff" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" text-rendering="geometricPrecision" font-size="110">
    <text x="{label_x}0" y="140" transform="scale(.1)" fill="#fff">{label}</text>
    <text x="{status_x}0" y="140" transform="scale(.1)" fill="#fff">{status}</text>
  </g>
</svg>"##
        )
    }

    fn plastic_badge(&self, label: &str, status: &str, color: &str) -> String {
        let label_width = self.text_width(label) + 10;
        let status_width = self.text_width(status) + 10;
        let total_width = label_width + status_width;
        let label_x = label_width / 2;
        let status_x = label_width + status_width / 2;

        format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="{total_width}" height="18" role="img" aria-label="{label}: {status}">
  <title>{label}: {status}</title>
  <linearGradient id="s" x2="0" y2="100%">
    <stop offset="0"  stop-color="#fff" stop-opacity=".7"/>
    <stop offset=".1" stop-color="#aaa" stop-opacity=".1"/>
    <stop offset=".9" stop-color="#000" stop-opacity=".3"/>
    <stop offset="1"  stop-color="#000" stop-opacity=".5"/>
  </linearGradient>
  <clipPath id="r">
    <rect width="{total_width}" height="18" rx="4" fill="#fff"/>
  </clipPath>
  <g clip-path="url(#r)">
    <rect width="{label_width}" height="18" fill="#555"/>
    <rect x="{label_width}" width="{status_width}" height="18" fill="{color}"/>
    <rect width="{total_width}" height="18" fill="url(#s)"/>
  </g>
  <g fill="#fff" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" text-rendering="geometricPrecision" font-size="110">
    <text x="{label_x}0" y="130" transform="scale(.1)" fill="#fff">{label}</text>
    <text x="{status_x}0" y="130" transform="scale(.1)" fill="#fff">{status}</text>
  </g>
</svg>"##
        )
    }

    fn for_the_badge(&self, label: &str, status: &str, color: &str) -> String {
        let label_upper = label.to_uppercase();
        let status_upper = status.to_uppercase();

        let label_width = self.text_width_large(&label_upper) + 20;
        let status_width = self.text_width_large(&status_upper) + 20;
        let total_width = label_width + status_width;
        let label_x = label_width / 2;
        let status_x = label_width + status_width / 2;
        let label_text_width = label_width - 20;
        let status_text_width = status_width - 20;

        format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="{total_width}" height="28" role="img" aria-label="{label}: {status}">
  <title>{label}: {status}</title>
  <g shape-rendering="crispEdges">
    <rect width="{label_width}" height="28" fill="#555"/>
    <rect x="{label_width}" width="{status_width}" height="28" fill="{color}"/>
  </g>
  <g fill="#fff" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" text-rendering="geometricPrecision" font-size="100">
    <text x="{label_x}0" y="175" transform="scale(.1)" fill="#fff" textLength="{label_text_width}0">{label_upper}</text>
    <text x="{status_x}0" y="175" transform="scale(.1)" fill="#fff" textLength="{status_text_width}0">{status_upper}</text>
  </g>
</svg>"##
        )
    }

    fn text_width(&self, text: &str) -> usize {
        // Approximate character width for 11px Verdana
        text.len() * 6 + 4
    }

    fn text_width_large(&self, text: &str) -> usize {
        // Approximate character width for larger text
        text.len() * 8 + 4
    }
}

impl Default for BadgeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate shields.io compatible URL
pub fn shields_io_url(report: &ComplianceReport) -> String {
    let (message, color) = match report.level {
        ComplianceLevel::Excellent => ("excellent", "brightgreen"),
        ComplianceLevel::Good => ("good", "green"),
        ComplianceLevel::Basic => ("basic", "yellow"),
        ComplianceLevel::NonCompliant => ("non--compliant", "red"),
    };

    let score = format!("{:.0}%25", report.score * 100.0);

    format!(
        "https://img.shields.io/badge/RSR-{}%20({})-{}",
        message, score, color
    )
}

/// Generate markdown badge
pub fn markdown_badge(report: &ComplianceReport, link: Option<&str>) -> String {
    let url = shields_io_url(report);
    let alt = format!("RSR Compliance: {:?}", report.level);

    if let Some(link) = link {
        format!("[![{}]({})]({})", alt, url, link)
    } else {
        format!("![{}]({})", alt, url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rsr::compliance::ComplianceStats;

    fn sample_report(level: ComplianceLevel, score: f64) -> ComplianceReport {
        ComplianceReport {
            level,
            score,
            requirements: vec![],
            stats: ComplianceStats::default(),
        }
    }

    #[test]
    fn test_generate_badge() {
        let generator = BadgeGenerator::new();
        let report = sample_report(ComplianceLevel::Excellent, 0.95);

        let svg = generator.generate(&report);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("RSR"));
        assert!(svg.contains("excellent"));
    }

    #[test]
    fn test_shields_io_url() {
        let report = sample_report(ComplianceLevel::Good, 0.85);
        let url = shields_io_url(&report);

        assert!(url.contains("shields.io"));
        assert!(url.contains("good"));
        assert!(url.contains("85%25"));
    }

    #[test]
    fn test_markdown_badge() {
        let report = sample_report(ComplianceLevel::Basic, 0.65);
        let md = markdown_badge(&report, Some("https://example.com"));

        assert!(md.contains("!["));
        assert!(md.contains("]("));
        assert!(md.contains("https://example.com"));
    }

    #[test]
    fn test_badge_styles() {
        let generator = BadgeGenerator::new();
        let report = sample_report(ComplianceLevel::Excellent, 0.95);

        // Test all styles generate valid SVG
        for style in [BadgeStyle::Flat, BadgeStyle::FlatSquare, BadgeStyle::Plastic, BadgeStyle::ForTheBadge] {
            let gen = generator.clone().style(style);
            let svg = gen.generate(&report);
            assert!(svg.contains("<svg"), "Style {:?} should generate SVG", style);
        }
    }
}
