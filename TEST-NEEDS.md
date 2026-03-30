# Test & Benchmark Requirements

## Current State
- Unit tests: BUILD FAILS — cargo test cannot complete (1 compilation error + 22 warnings)
- Integration tests: 1 Zig integration test (template)
- E2E tests: NONE
- Benchmarks: NONE
- panic-attack scan: NEVER RUN

## What's Missing
### Point-to-Point (P2P)
45 Rust source files with 22 files containing #[cfg(test)] / #[test] — but none can run because the build is broken.

#### Modules (organized by subsystem):
**Analyzer (5 files):**
- complexity.rs — has inline tests (BLOCKED)
- config_detector.rs — has inline tests (BLOCKED)
- patterns.rs — has inline tests (BLOCKED)
- recommender.rs — likely has tests (BLOCKED)
- mod.rs

**Cache (3 files):**
- filesystem.rs — needs tests
- hash.rs — needs tests
- mod.rs

**CLI (8 files):**
- analyze.rs, cache.rs, graph.rs, init.rs, run.rs, validate.rs, watch.rs, rsr.rs
- CLI commands need integration tests

**Errors (3 files):**
- educational.rs — error message quality tests
- recovery.rs — recovery path tests
- mod.rs

**Executors (4 files):**
- cue.rs, nickel.rs, shell.rs — executor correctness tests
- mod.rs

**Pipeline (5 files):**
- dag.rs — DAG construction and cycle detection tests
- definition.rs — pipeline definition parsing tests
- executor.rs — pipeline execution tests
- validation.rs — **THIS IS WHERE THE BUILD ERROR IS** (unused field `flags`, `shell`)
- mod.rs

**RSR (multiple files):**
- badges.rs + others — RSR generation tests

### End-to-End (E2E)
- Full pipeline: define -> validate -> execute -> cache results
- CLI: init project -> define pipeline -> run -> watch for changes
- Analyzer: scan project -> detect configs -> recommend pipeline
- Cache: execute -> cache -> re-execute (cache hit)
- Graph generation and visualization
- Error recovery: introduce error -> recover -> resume

### Aspect Tests
- [ ] Security (shell executor command injection, path traversal in cache, untrusted pipeline definitions)
- [ ] Performance (DAG resolution on large pipelines, cache lookup speed)
- [ ] Concurrency (parallel pipeline stages, file watching races)
- [ ] Error handling (executor failures, missing tools, invalid configs)
- [ ] Accessibility (N/A — CLI tool)

### Build & Execution
- [ ] cargo build — **FAILS** (compilation error in pipeline/validation.rs)
- [ ] cargo test — **FAILS** (blocked by build error)
- [ ] Binary runs — **BLOCKED**
- [ ] CLI --help — **BLOCKED**
- [ ] Self-diagnostic — none

### Benchmarks Needed
- Pipeline execution throughput
- DAG resolution for complex dependency graphs
- Cache hit/miss ratio under typical workloads
- File watching latency
- Analyzer scan speed on large projects

### Self-Tests
- [ ] panic-attack assail on own repo
- [ ] Fix compilation error first (pipeline/validation.rs)
- [ ] Built-in doctor/check command (if applicable)

## Priority
- **HIGH** — 45 Rust source files that CANNOT EVEN COMPILE. The build is broken. ~20 files have inline test modules but none can run. Fix the compilation error in pipeline/validation.rs first, then verify test coverage. A configuration flow tool with shell execution needs security testing as a top priority.
