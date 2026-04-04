// SPDX-License-Identifier: PMPL-1.0-or-later
// benches/conflow_bench.rs — Criterion benchmarks for the conflow analyzer.
//
// Benchmarks cover three payload sizes:
//   small  — a few-line YAML service definition (~5 lines)
//   medium — a realistic Kubernetes Deployment (~40 lines)
//   large  — a dense JSON configuration (~100 key-value pairs)
//
// Run with:
//   cargo bench --bench conflow_bench

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Synthetic config payloads
// ---------------------------------------------------------------------------

/// Small: a minimal YAML service config.
const SMALL_YAML: &str = "\
name: svc\n\
port: 8080\n\
debug: false\n\
replicas: 1\n\
log_level: info\n";

/// Medium: a realistic Kubernetes Deployment manifest.
const MEDIUM_YAML: &str = "\
apiVersion: apps/v1\n\
kind: Deployment\n\
metadata:\n\
  name: my-service\n\
  namespace: default\n\
  labels:\n\
    app: my-service\n\
    tier: backend\n\
spec:\n\
  replicas: 3\n\
  selector:\n\
    matchLabels:\n\
      app: my-service\n\
  template:\n\
    metadata:\n\
      labels:\n\
        app: my-service\n\
    spec:\n\
      containers:\n\
        - name: app\n\
          image: my-service:latest\n\
          ports:\n\
            - containerPort: 8080\n\
          env:\n\
            - name: LOG_LEVEL\n\
              value: info\n\
            - name: DB_HOST\n\
              value: postgres\n\
          resources:\n\
            requests:\n\
              cpu: 100m\n\
              memory: 128Mi\n\
            limits:\n\
              cpu: 500m\n\
              memory: 512Mi\n\
          readinessProbe:\n\
            httpGet:\n\
              path: /health\n\
              port: 8080\n\
            initialDelaySeconds: 5\n\
            periodSeconds: 10\n";

/// Large: a JSON config with 100 key-value pairs.
fn large_json() -> String {
    let mut s = String::from("{\n");
    for i in 0..100 {
        let comma = if i < 99 { "," } else { "" };
        s.push_str(&format!(
            "  \"service_{}\": {{ \"port\": {}, \"enabled\": true, \"weight\": {} }}{}\n",
            i,
            8000 + i,
            i * 10,
            comma
        ));
    }
    s.push_str("}\n");
    s
}

// ---------------------------------------------------------------------------
// Benchmark helpers
// ---------------------------------------------------------------------------

/// Write content to a fixed temp path and return the path.
///
/// NOTE: Uses a hard-coded /tmp path to avoid TempDir overhead inside the
/// hot loop.  Files are small and overwritten each bench run — safe for CI.
fn write_bench_file(name: &str, content: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("conflow_bench_{}", name));
    std::fs::write(&path, content).expect("write bench file");
    path
}

/// Run an async future synchronously (single-threaded runtime).
fn run_sync<F: std::future::Future>(fut: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build runtime")
        .block_on(fut)
}

// ---------------------------------------------------------------------------
// Benchmarks
// ---------------------------------------------------------------------------

fn bench_analyzer_small(c: &mut Criterion) {
    let path = write_bench_file("small.yaml", SMALL_YAML);
    let analyzer = conflow::analyzer::ConfigAnalyzer::new();

    c.bench_function("analyzer/small_yaml", |b| {
        b.iter(|| {
            let result = run_sync(analyzer.analyze(black_box(&path)));
            black_box(result.expect("analyze small"));
        })
    });
}

fn bench_analyzer_medium(c: &mut Criterion) {
    let path = write_bench_file("medium.yaml", MEDIUM_YAML);
    let analyzer = conflow::analyzer::ConfigAnalyzer::new();

    c.bench_function("analyzer/medium_yaml_k8s", |b| {
        b.iter(|| {
            let result = run_sync(analyzer.analyze(black_box(&path)));
            black_box(result.expect("analyze medium"));
        })
    });
}

fn bench_analyzer_large(c: &mut Criterion) {
    let json = large_json();
    let path = write_bench_file("large.json", &json);
    let analyzer = conflow::analyzer::ConfigAnalyzer::new();

    c.bench_function("analyzer/large_json_100_services", |b| {
        b.iter(|| {
            let result = run_sync(analyzer.analyze(black_box(&path)));
            black_box(result.expect("analyze large"));
        })
    });
}

fn bench_pipeline_parse_small(c: &mut Criterion) {
    let yaml = "\
version: \"1\"\n\
name: bench-pipeline\n\
stages:\n\
  - name: validate\n\
    tool:\n\
      type: cue\n\
      command: vet\n\
    input: \"*.json\"\n";

    c.bench_function("pipeline/parse_small_yaml", |b| {
        b.iter(|| {
            let pipeline = conflow::pipeline::Pipeline::from_yaml(black_box(yaml));
            black_box(pipeline.expect("parse pipeline"));
        })
    });
}

criterion_group!(
    benches,
    bench_analyzer_small,
    bench_analyzer_medium,
    bench_analyzer_large,
    bench_pipeline_parse_small,
);
criterion_main!(benches);
