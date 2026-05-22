// SPDX-License-Identifier: MPL-2.0
// tests/e2e_test.rs — End-to-end tests for the conflow analyzer and pipeline.
//
// These tests exercise the public API against realistic config files written
// into temporary directories, verifying that the full analysis chain works
// from disk I/O through format detection, complexity scoring, and tool
// recommendation.
//
// No external test crate dependencies are used beyond those already declared
// in Cargo.toml (tempfile = "3.9" is a dev-dependency).

use std::path::PathBuf;

use conflow::analyzer::{ConfigAnalyzer, ConfigFormat};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Write a file into a tempdir and return its path.
fn write_temp(dir: &tempfile::TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, content).expect("write_temp failed");
    path
}

/// Block on an async future using a single-threaded tokio runtime.
fn run<F: std::future::Future>(fut: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build runtime")
        .block_on(fut)
}

// ---------------------------------------------------------------------------
// 1. JSON file detected and analyzed
// ---------------------------------------------------------------------------

#[test]
fn e2e_json_simple_config() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = write_temp(&dir, "config.json", r#"{"name": "test", "value": 42}"#);

    let analyzer = ConfigAnalyzer::new();
    let analysis = run(analyzer.analyze(&path)).expect("analyze should succeed");

    assert_eq!(analysis.format, ConfigFormat::Json, "format should be JSON");
    assert!(
        analysis.complexity.line_count >= 1,
        "must have at least one line"
    );
}

// ---------------------------------------------------------------------------
// 2. YAML file detected and analyzed
// ---------------------------------------------------------------------------

#[test]
fn e2e_yaml_config() {
    let dir = tempfile::TempDir::new().unwrap();
    let content = "name: my-service\nport: 8080\nenabled: true\n";
    let path = write_temp(&dir, "config.yaml", content);

    let analyzer = ConfigAnalyzer::new();
    let analysis = run(analyzer.analyze(&path)).expect("analyze should succeed");

    assert_eq!(analysis.format, ConfigFormat::Yaml, "format should be YAML");
}

// ---------------------------------------------------------------------------
// 3. TOML file detected and analyzed
// ---------------------------------------------------------------------------

#[test]
fn e2e_toml_config() {
    let dir = tempfile::TempDir::new().unwrap();
    let content = "[package]\nname = \"foo\"\nversion = \"0.1.0\"\n";
    let path = write_temp(&dir, "Cargo.toml", content);

    let analyzer = ConfigAnalyzer::new();
    let analysis = run(analyzer.analyze(&path)).expect("analyze should succeed");

    assert_eq!(analysis.format, ConfigFormat::Toml, "format should be TOML");
}

// ---------------------------------------------------------------------------
// 4. Nickel file detected and analyzed
// ---------------------------------------------------------------------------

#[test]
fn e2e_nickel_simple() {
    let dir = tempfile::TempDir::new().unwrap();
    let content = "let name = \"service\" in { name = name, port = 8080 }\n";
    let path = write_temp(&dir, "config.ncl", content);

    let analyzer = ConfigAnalyzer::new();
    let analysis = run(analyzer.analyze(&path)).expect("analyze should succeed");

    assert_eq!(
        analysis.format,
        ConfigFormat::Nickel,
        "format should be Nickel"
    );
}

// ---------------------------------------------------------------------------
// 5. CUE file detected and analyzed
// ---------------------------------------------------------------------------

#[test]
fn e2e_cue_schema() {
    let dir = tempfile::TempDir::new().unwrap();
    let content = "#Config: {\n    name:  string\n    port:  int & >=1 & <=65535\n}\n";
    let path = write_temp(&dir, "schema.cue", content);

    let analyzer = ConfigAnalyzer::new();
    let analysis = run(analyzer.analyze(&path)).expect("analyze should succeed");

    assert_eq!(analysis.format, ConfigFormat::Cue, "format should be CUE");
}

// ---------------------------------------------------------------------------
// 6. Logic-heavy Nickel recommends Nickel tool
// ---------------------------------------------------------------------------

#[test]
fn e2e_nickel_with_logic_recommends_nickel() {
    let dir = tempfile::TempDir::new().unwrap();
    let content = r#"
let make_config = fun env =>
  if env == "prod" then {
    replicas = 3,
    debug    = false,
  } else {
    replicas = 1,
    debug    = true,
  }
in make_config "dev"
"#;
    let path = write_temp(&dir, "app.ncl", content);

    let analyzer = ConfigAnalyzer::new();
    let analysis = run(analyzer.analyze(&path)).expect("analyze should succeed");

    // Logic + function patterns should steer recommendation toward Nickel.
    use conflow::analyzer::RecommendedTool;
    assert_eq!(
        analysis.recommendation.primary,
        RecommendedTool::Nickel,
        "logic-heavy Nickel should recommend Nickel"
    );
}

// ---------------------------------------------------------------------------
// 7. Constraint-heavy CUE recommends CUE tool
// ---------------------------------------------------------------------------

#[test]
fn e2e_cue_with_constraints_recommends_cue() {
    let dir = tempfile::TempDir::new().unwrap();
    let content = r#"
#Port: int & >=1 & <=65535
#Config: {
    host:    string
    port:    #Port
    timeout: int & >=100 & <=30000
}
"#;
    let path = write_temp(&dir, "constraints.cue", content);

    let analyzer = ConfigAnalyzer::new();
    let analysis = run(analyzer.analyze(&path)).expect("analyze should succeed");

    use conflow::analyzer::RecommendedTool;
    assert_eq!(
        analysis.recommendation.primary,
        RecommendedTool::Cue,
        "constraint-heavy CUE should recommend CUE"
    );
}

// ---------------------------------------------------------------------------
// 8. Pipeline round-trip: parse YAML, validate structure
// ---------------------------------------------------------------------------

#[test]
fn e2e_pipeline_roundtrip_from_yaml() {
    use conflow::pipeline::{Pipeline, PipelineValidator};

    let yaml = r#"
version: "1"
name: test-pipeline
stages:
  - name: validate
    tool:
      type: cue
      command: vet
    input: "*.json"
"#;

    let pipeline = Pipeline::from_yaml(yaml).expect("pipeline should parse");
    assert_eq!(pipeline.name, "test-pipeline");
    assert_eq!(pipeline.stages.len(), 1);

    let result = PipelineValidator::validate(&pipeline).expect("validate should not error");
    assert!(result.is_valid(), "pipeline should be valid: {:?}", result.errors);
}

// ---------------------------------------------------------------------------
// 9. Pipeline with duplicate stage names fails validation
// ---------------------------------------------------------------------------

#[test]
fn e2e_pipeline_duplicate_stage_name_invalid() {
    use conflow::pipeline::{Pipeline, PipelineValidator};

    let yaml = r#"
version: "1"
name: dup-test
stages:
  - name: step
    tool:
      type: shell
      command: "echo hello"
    input: "*.json"
  - name: step
    tool:
      type: shell
      command: "echo world"
    input: "*.yaml"
"#;

    let pipeline = Pipeline::from_yaml(yaml).expect("pipeline should parse");
    let result = PipelineValidator::validate(&pipeline).expect("validate returns Ok");
    assert!(
        !result.is_valid(),
        "duplicate stage names must be detected as invalid"
    );
    assert!(
        result.errors.iter().any(|e| e.contains("Duplicate")),
        "error should mention Duplicate"
    );
}

// ---------------------------------------------------------------------------
// 10. Analyzing a missing file returns an error (not a panic)
// ---------------------------------------------------------------------------

#[test]
fn e2e_missing_file_returns_error() {
    let path = PathBuf::from("/tmp/conflow_definitely_does_not_exist_xyzzy.json");
    let analyzer = ConfigAnalyzer::new();
    let result = run(analyzer.analyze(&path));
    assert!(
        result.is_err(),
        "analyzing a missing file must return an error"
    );
}

// ---------------------------------------------------------------------------
// 11. Large JSON config has correct line count
// ---------------------------------------------------------------------------

#[test]
fn e2e_large_json_line_count() {
    let dir = tempfile::TempDir::new().unwrap();
    // Build a JSON object with 30 key-value pairs (one per line).
    let mut content = "{\n".to_string();
    for i in 0..30 {
        let comma = if i < 29 { "," } else { "" };
        content.push_str(&format!("  \"key{}\": {}{}\n", i, i, comma));
    }
    content.push_str("}\n");

    let path = write_temp(&dir, "big.json", &content);
    let analyzer = ConfigAnalyzer::new();
    let analysis = run(analyzer.analyze(&path)).expect("analyze should succeed");

    assert!(
        analysis.complexity.line_count >= 30,
        "line count should reflect actual file length"
    );
}

// ---------------------------------------------------------------------------
// 12. Nickel with std.map detected as having generation patterns
// ---------------------------------------------------------------------------

#[test]
fn e2e_nickel_generation_patterns_detected() {
    let dir = tempfile::TempDir::new().unwrap();
    let content =
        "let xs = std.map (fun x => x * 2) [1, 2, 3, 4, 5] in { values = xs }\n";
    let path = write_temp(&dir, "gen.ncl", content);

    let analyzer = ConfigAnalyzer::new();
    let analysis = run(analyzer.analyze(&path)).expect("analyze should succeed");

    assert!(
        analysis.complexity.has_generation || analysis.complexity.has_functions,
        "std.map / fun should be detected as generation or function pattern"
    );
}
