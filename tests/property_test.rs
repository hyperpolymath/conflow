// SPDX-License-Identifier: PMPL-1.0-or-later
// tests/property_test.rs — Property-style tests for the conflow analyzer.
//
// These tests verify two core properties:
//   P1. DETERMINISM  — the same input always produces the same analysis result.
//   P2. ERROR POLICY — inputs that should be invalid trigger errors, not panics.
//
// A fixed array of representative inputs is used in place of a generative
// property-testing library (no new dev-dependencies required).

use std::path::PathBuf;

use conflow::analyzer::{ConfigAnalyzer, ConfigFormat};

// ---------------------------------------------------------------------------
// Shared runtime helper
// ---------------------------------------------------------------------------

fn run<F: std::future::Future>(fut: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build runtime")
        .block_on(fut)
}

fn write_temp(dir: &tempfile::TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, content).expect("write_temp");
    path
}

// ---------------------------------------------------------------------------
// P1. DETERMINISM — same input → same output (run twice, compare)
// ---------------------------------------------------------------------------

/// A representative set of config snippets paired with their expected format.
struct TestInput {
    filename: &'static str,
    content: &'static str,
    expected_format: ConfigFormat,
}

const DETERMINISM_CASES: &[TestInput] = &[
    TestInput {
        filename: "simple.json",
        content: r#"{"app": "conflow", "version": 1}"#,
        expected_format: ConfigFormat::Json,
    },
    TestInput {
        filename: "service.yaml",
        content: "name: svc\nport: 8080\nreplicas: 2\n",
        expected_format: ConfigFormat::Yaml,
    },
    TestInput {
        filename: "manifest.toml",
        content: "[server]\nhost = \"localhost\"\nport = 9000\n",
        expected_format: ConfigFormat::Toml,
    },
    TestInput {
        filename: "schema.cue",
        content: "#Config: { host: string, port: int & >=1 }\n",
        expected_format: ConfigFormat::Cue,
    },
    TestInput {
        filename: "app.ncl",
        content: "let port = 3000 in { host = \"localhost\", port = port }\n",
        expected_format: ConfigFormat::Nickel,
    },
    TestInput {
        filename: "logic.ncl",
        content: "let f = fun x => if x > 0 then \"pos\" else \"neg\" in { result = f 5 }\n",
        expected_format: ConfigFormat::Nickel,
    },
    TestInput {
        filename: "deep.json",
        content: r#"{"a":{"b":{"c":{"d":{"e":1}}}}}"#,
        expected_format: ConfigFormat::Json,
    },
    TestInput {
        filename: "constraints.cue",
        content: "#V: int & >=0 & <=100\n#S: { v: #V, label: string & =~\"^[a-z]+$\" }\n",
        expected_format: ConfigFormat::Cue,
    },
    TestInput {
        filename: "multi.yaml",
        content: "---\nkind: Deployment\nmetadata:\n  name: app\nspec:\n  replicas: 3\n",
        expected_format: ConfigFormat::Yaml,
    },
    TestInput {
        filename: "packages.toml",
        content: "[dependencies]\nserde = \"1.0\"\ntokio = { version = \"1\", features = [\"full\"] }\n",
        expected_format: ConfigFormat::Toml,
    },
];

#[test]
fn property_determinism_same_input_same_format() {
    let dir = tempfile::TempDir::new().unwrap();
    let analyzer = ConfigAnalyzer::new();

    for case in DETERMINISM_CASES {
        let path = write_temp(&dir, case.filename, case.content);

        // First run
        let result1 = run(analyzer.analyze(&path)).expect("first analyze");
        // Second run (same file, same analyzer instance)
        let result2 = run(analyzer.analyze(&path)).expect("second analyze");

        assert_eq!(
            result1.format, result2.format,
            "format must be deterministic for '{}'",
            case.filename
        );
        assert_eq!(
            result1.complexity.line_count, result2.complexity.line_count,
            "line_count must be deterministic for '{}'",
            case.filename
        );
        assert_eq!(
            result1.complexity.nesting_depth, result2.complexity.nesting_depth,
            "nesting_depth must be deterministic for '{}'",
            case.filename
        );
        assert_eq!(
            result1.complexity.has_logic, result2.complexity.has_logic,
            "has_logic must be deterministic for '{}'",
            case.filename
        );
        assert_eq!(
            result1.recommendation.primary, result2.recommendation.primary,
            "recommendation must be deterministic for '{}'",
            case.filename
        );
    }
}

#[test]
fn property_determinism_expected_format() {
    let dir = tempfile::TempDir::new().unwrap();
    let analyzer = ConfigAnalyzer::new();

    for case in DETERMINISM_CASES {
        let path = write_temp(&dir, case.filename, case.content);
        let analysis = run(analyzer.analyze(&path)).expect("analyze");
        assert_eq!(
            analysis.format, case.expected_format,
            "format for '{}' should be {:?}",
            case.filename, case.expected_format
        );
    }
}

// ---------------------------------------------------------------------------
// P2. ERROR POLICY — non-existent paths must produce errors, not panics
// ---------------------------------------------------------------------------

/// A set of paths that do not exist on disk.
const INVALID_PATHS: &[&str] = &[
    "/tmp/conflow_no_such_file_aaa.json",
    "/tmp/conflow_no_such_file_bbb.yaml",
    "/tmp/conflow_no_such_file_ccc.toml",
    "/tmp/conflow_no_such_file_ddd.cue",
    "/tmp/conflow_no_such_file_eee.ncl",
    "/tmp/conflow_no_such_dir/nested/config.json",
    "/root/forbidden_conflow_test.json",
    "/nonexistent/deep/path/config.yaml",
];

#[test]
fn property_invalid_paths_return_errors() {
    let analyzer = ConfigAnalyzer::new();

    for path_str in INVALID_PATHS {
        let path = PathBuf::from(path_str);
        let result = run(analyzer.analyze(&path));
        assert!(
            result.is_err(),
            "analyzing non-existent '{}' must return Err, not Ok",
            path_str
        );
    }
}

// ---------------------------------------------------------------------------
// P3. CONSISTENCY — line_count matches actual newline count for all inputs
// ---------------------------------------------------------------------------

#[test]
fn property_line_count_matches_actual_newlines() {
    let dir = tempfile::TempDir::new().unwrap();
    let analyzer = ConfigAnalyzer::new();

    // Build a set of inputs with known line counts.
    let cases: &[(&str, &str)] = &[
        ("one.json", r#"{"x":1}"#),                           // 1 line (no newline)
        ("two.yaml", "a: 1\nb: 2\n"),                         // 2 lines
        ("five.toml", "[s]\na=1\nb=2\nc=3\nd=4\n"),           // 5 lines
        ("ten.ncl", "let f = fun x => x in {\n  a=f 1,\n  b=f 2,\n  c=f 3,\n  d=f 4,\n  e=f 5,\n  g=f 6,\n  h=f 7,\n  i=f 8,\n  j=f 9\n}\n"),
    ];

    for (filename, content) in cases {
        let actual_lines = content.lines().count();
        let path = write_temp(&dir, filename, content);
        let analysis = run(analyzer.analyze(&path)).expect("analyze");
        assert_eq!(
            analysis.complexity.line_count, actual_lines,
            "line_count for '{}' should be {}, got {}",
            filename, actual_lines, analysis.complexity.line_count
        );
    }
}
