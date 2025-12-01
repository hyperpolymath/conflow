//! RSR Hooks - Integration points for RSR validator
//!
//! Provides hooks that RSR validator can use to trigger conflow operations
//! and receive results.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::pipeline::{ExecutionOptions, Pipeline, PipelineExecutor, PipelineResult};
use crate::executors::create_default_executors;
use crate::cache::FilesystemCache;
use crate::ConflowError;

/// Trigger types for RSR integration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RsrTrigger {
    /// Validate pipeline configuration
    ValidatePipeline {
        path: PathBuf,
    },

    /// Run pipeline
    RunPipeline {
        path: PathBuf,
        stages: Vec<String>,
        no_cache: bool,
    },

    /// Check compliance
    CheckCompliance {
        requirements: Vec<String>,
    },

    /// Initialize from template
    InitFromTemplate {
        template: String,
        target_dir: PathBuf,
    },

    /// Analyze configuration file
    AnalyzeConfig {
        file: PathBuf,
    },
}

/// Result of an RSR hook execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsrHookResult {
    /// Whether the hook succeeded
    pub success: bool,

    /// Result message
    pub message: String,

    /// Detailed data (JSON-serializable)
    pub data: Option<serde_json::Value>,

    /// Suggestions for next steps
    pub suggestions: Vec<String>,
}

impl RsrHookResult {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: None,
            suggestions: vec![],
        }
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            data: None,
            suggestions: vec![],
        }
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions = suggestions;
        self
    }
}

/// RSR Hooks handler
pub struct RsrHooks {
    working_dir: PathBuf,
}

impl RsrHooks {
    /// Create a new hooks handler
    pub fn new(working_dir: PathBuf) -> Self {
        Self { working_dir }
    }

    /// Execute a trigger
    pub async fn execute(&self, trigger: RsrTrigger) -> RsrHookResult {
        match trigger {
            RsrTrigger::ValidatePipeline { path } => {
                self.validate_pipeline(&path).await
            }
            RsrTrigger::RunPipeline { path, stages, no_cache } => {
                self.run_pipeline(&path, stages, no_cache).await
            }
            RsrTrigger::CheckCompliance { requirements } => {
                self.check_compliance(&requirements).await
            }
            RsrTrigger::InitFromTemplate { template, target_dir } => {
                self.init_from_template(&template, &target_dir).await
            }
            RsrTrigger::AnalyzeConfig { file } => {
                self.analyze_config(&file).await
            }
        }
    }

    /// Validate a pipeline configuration
    async fn validate_pipeline(&self, path: &Path) -> RsrHookResult {
        let full_path = self.working_dir.join(path);

        match Pipeline::from_file(&full_path) {
            Ok(pipeline) => {
                match crate::pipeline::PipelineValidator::validate(&pipeline) {
                    Ok(validation) => {
                        if validation.is_valid() {
                            RsrHookResult::success("Pipeline is valid")
                                .with_data(serde_json::json!({
                                    "name": pipeline.name,
                                    "stages": pipeline.stages.len(),
                                    "warnings": validation.warnings,
                                }))
                        } else {
                            RsrHookResult::failure("Pipeline validation failed")
                                .with_data(serde_json::json!({
                                    "errors": validation.errors,
                                    "warnings": validation.warnings,
                                }))
                                .with_suggestions(vec![
                                    "Check stage dependencies".into(),
                                    "Verify tool configurations".into(),
                                    "Run 'conflow validate' for details".into(),
                                ])
                        }
                    }
                    Err(e) => RsrHookResult::failure(format!("Validation error: {}", e)),
                }
            }
            Err(e) => RsrHookResult::failure(format!("Failed to load pipeline: {}", e))
                .with_suggestions(vec![
                    "Run 'conflow init' to create a pipeline".into(),
                    "Check YAML syntax".into(),
                ]),
        }
    }

    /// Run a pipeline
    async fn run_pipeline(
        &self,
        path: &Path,
        stages: Vec<String>,
        no_cache: bool,
    ) -> RsrHookResult {
        let full_path = self.working_dir.join(path);

        let pipeline = match Pipeline::from_file(&full_path) {
            Ok(p) => p,
            Err(e) => return RsrHookResult::failure(format!("Failed to load pipeline: {}", e)),
        };

        let mut executor = PipelineExecutor::new();
        for (name, exec) in create_default_executors() {
            executor.register_executor(&name, exec);
        }

        if !no_cache && pipeline.cache.enabled {
            if let Ok(cache) = FilesystemCache::new(
                self.working_dir.join(&pipeline.cache.directory),
                self.working_dir.clone(),
            ) {
                executor = executor.with_cache(Box::new(cache));
            }
        }

        let options = ExecutionOptions {
            no_cache,
            dry_run: false,
            stages,
            verbose: false,
        };

        match executor.execute(&pipeline, &self.working_dir, &options).await {
            Ok(result) => {
                let outputs: Vec<String> = result
                    .results
                    .values()
                    .flat_map(|r| r.outputs.iter())
                    .map(|p| p.display().to_string())
                    .collect();

                if result.success {
                    RsrHookResult::success("Pipeline completed successfully")
                        .with_data(serde_json::json!({
                            "duration_ms": result.duration.as_millis(),
                            "stages_run": result.results.len(),
                            "outputs": outputs,
                        }))
                } else {
                    let failed: Vec<String> = result
                        .results
                        .iter()
                        .filter(|(_, r)| !r.success)
                        .map(|(name, _)| name.clone())
                        .collect();

                    RsrHookResult::failure("Pipeline failed")
                        .with_data(serde_json::json!({
                            "failed_stages": failed,
                            "duration_ms": result.duration.as_millis(),
                        }))
                }
            }
            Err(e) => RsrHookResult::failure(format!("Pipeline execution failed: {}", e)),
        }
    }

    /// Check RSR compliance
    async fn check_compliance(&self, requirements: &[String]) -> RsrHookResult {
        use super::compliance::ComplianceChecker;

        let checker = ComplianceChecker::new();

        if requirements.is_empty() {
            // Check all requirements
            match checker.check(&self.working_dir) {
                Ok(report) => {
                    RsrHookResult::success(format!(
                        "Compliance: {} ({:.0}%)",
                        report.level.description(),
                        report.score * 100.0
                    ))
                    .with_data(serde_json::json!({
                        "level": format!("{:?}", report.level),
                        "score": report.score,
                        "stats": {
                            "total": report.stats.total,
                            "passed": report.stats.passed,
                            "failed": report.stats.failed,
                        },
                        "requirements": report.requirements.iter().map(|r| {
                            serde_json::json!({
                                "id": r.requirement_id,
                                "met": r.met,
                            })
                        }).collect::<Vec<_>>(),
                    }))
                }
                Err(e) => RsrHookResult::failure(format!("Compliance check failed: {}", e)),
            }
        } else {
            // Check specific requirements
            let req_refs: Vec<&str> = requirements.iter().map(|s| s.as_str()).collect();
            match checker.check_requirements(&req_refs, &self.working_dir) {
                Ok(results) => {
                    let all_met = results.iter().all(|r| r.met);
                    let message = if all_met {
                        "All checked requirements met".to_string()
                    } else {
                        format!(
                            "{}/{} requirements met",
                            results.iter().filter(|r| r.met).count(),
                            results.len()
                        )
                    };

                    RsrHookResult::success(message)
                        .with_data(serde_json::json!({
                            "requirements": results.iter().map(|r| {
                                serde_json::json!({
                                    "id": r.requirement_id,
                                    "met": r.met,
                                    "remediation": r.remediation,
                                })
                            }).collect::<Vec<_>>(),
                        }))
                }
                Err(e) => RsrHookResult::failure(format!("Compliance check failed: {}", e)),
            }
        }
    }

    /// Initialize from a template
    async fn init_from_template(&self, template: &str, target_dir: &Path) -> RsrHookResult {
        // This would call the init command logic
        // For now, return a placeholder
        RsrHookResult::success(format!("Would initialize '{}' template in {}", template, target_dir.display()))
            .with_suggestions(vec![
                format!("Run: cd {} && conflow init --template {}", target_dir.display(), template),
            ])
    }

    /// Analyze a configuration file
    async fn analyze_config(&self, file: &Path) -> RsrHookResult {
        use crate::analyzer::ConfigAnalyzer;

        let full_path = self.working_dir.join(file);
        let analyzer = ConfigAnalyzer::new();

        match analyzer.analyze(&full_path).await {
            Ok(analysis) => {
                RsrHookResult::success(format!(
                    "Recommended tool: {:?}",
                    analysis.recommendation.primary
                ))
                .with_data(serde_json::json!({
                    "format": format!("{:?}", analysis.format),
                    "complexity": {
                        "has_logic": analysis.complexity.has_logic,
                        "has_functions": analysis.complexity.has_functions,
                        "has_constraints": analysis.complexity.has_constraints,
                        "nesting_depth": analysis.complexity.nesting_depth,
                    },
                    "recommendation": {
                        "primary": format!("{:?}", analysis.recommendation.primary),
                        "rationale": analysis.recommendation.rationale,
                        "combined_approach": analysis.recommendation.combined_approach,
                    },
                }))
            }
            Err(e) => RsrHookResult::failure(format!("Analysis failed: {}", e)),
        }
    }
}

/// JSON-RPC style interface for external integration
pub mod rpc {
    use super::*;
    use serde::{Deserialize, Serialize};

    /// RPC Request
    #[derive(Debug, Serialize, Deserialize)]
    pub struct RpcRequest {
        pub jsonrpc: String,
        pub method: String,
        pub params: serde_json::Value,
        pub id: serde_json::Value,
    }

    /// RPC Response
    #[derive(Debug, Serialize, Deserialize)]
    pub struct RpcResponse {
        pub jsonrpc: String,
        pub result: Option<serde_json::Value>,
        pub error: Option<RpcError>,
        pub id: serde_json::Value,
    }

    /// RPC Error
    #[derive(Debug, Serialize, Deserialize)]
    pub struct RpcError {
        pub code: i32,
        pub message: String,
        pub data: Option<serde_json::Value>,
    }

    impl RpcResponse {
        pub fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
            Self {
                jsonrpc: "2.0".into(),
                result: Some(result),
                error: None,
                id,
            }
        }

        pub fn error(id: serde_json::Value, code: i32, message: String) -> Self {
            Self {
                jsonrpc: "2.0".into(),
                result: None,
                error: Some(RpcError {
                    code,
                    message,
                    data: None,
                }),
                id,
            }
        }
    }

    /// Handle an RPC request
    pub async fn handle_request(
        hooks: &RsrHooks,
        request: RpcRequest,
    ) -> RpcResponse {
        let trigger = match request.method.as_str() {
            "conflow.validate" => {
                let path = request.params.get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".conflow.yaml");
                RsrTrigger::ValidatePipeline { path: PathBuf::from(path) }
            }
            "conflow.run" => {
                let path = request.params.get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".conflow.yaml");
                let stages = request.params.get("stages")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();
                let no_cache = request.params.get("no_cache")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                RsrTrigger::RunPipeline {
                    path: PathBuf::from(path),
                    stages,
                    no_cache,
                }
            }
            "conflow.compliance" => {
                let requirements = request.params.get("requirements")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();
                RsrTrigger::CheckCompliance { requirements }
            }
            "conflow.analyze" => {
                let file = request.params.get("file")
                    .and_then(|v| v.as_str())
                    .unwrap_or("config.yaml");
                RsrTrigger::AnalyzeConfig { file: PathBuf::from(file) }
            }
            _ => {
                return RpcResponse::error(
                    request.id,
                    -32601,
                    format!("Method not found: {}", request.method),
                );
            }
        };

        let result = hooks.execute(trigger).await;

        RpcResponse::success(
            request.id,
            serde_json::json!({
                "success": result.success,
                "message": result.message,
                "data": result.data,
                "suggestions": result.suggestions,
            }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_validate_missing_pipeline() {
        let temp = TempDir::new().unwrap();
        let hooks = RsrHooks::new(temp.path().to_path_buf());

        let result = hooks.execute(RsrTrigger::ValidatePipeline {
            path: PathBuf::from(".conflow.yaml"),
        }).await;

        assert!(!result.success);
        assert!(result.message.contains("Failed to load"));
    }

    #[tokio::test]
    async fn test_validate_valid_pipeline() {
        let temp = TempDir::new().unwrap();

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
        ).unwrap();

        let hooks = RsrHooks::new(temp.path().to_path_buf());

        let result = hooks.execute(RsrTrigger::ValidatePipeline {
            path: PathBuf::from(".conflow.yaml"),
        }).await;

        assert!(result.success);
    }
}
