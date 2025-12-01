//! RSR (Rhodium Standard Repository) Integration
//!
//! This module provides integration between conflow and the RSR ecosystem,
//! enabling:
//! - Validation of RSR requirement configurations
//! - Compliance checking for RSR-CONFIG-002
//! - Integration hooks for RSR validator
//! - Shared schema validation

pub mod compliance;
pub mod hooks;
pub mod requirements;
pub mod schemas;

pub use compliance::{
    CheckDetail, ComplianceChecker, ComplianceLevel, ComplianceReport, ComplianceStats,
    RequirementResult,
};
pub use hooks::{RsrHooks, RsrTrigger};
pub use requirements::{RsrRequirement, RsrRequirementClass, RsrRequirementRegistry};
pub use schemas::RsrSchemaRegistry;
