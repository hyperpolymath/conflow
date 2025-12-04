# CLAUDE.md - AI Assistant Guidance for conflow

This document provides guidance for AI assistants working with the conflow codebase.

## Project Overview

**conflow** is a Configuration Flow Orchestrator that intelligently orchestrates
CUE, Nickel, and configuration validation workflows.

### Key Concepts

- **Pipeline**: A sequence of stages defined in `.conflow.yaml`
- **Stage**: A single step that runs a tool (CUE, Nickel, or shell)
- **Executor**: Implements tool-specific execution logic
- **Cache**: Content-addressed caching to avoid redundant work
- **RSR Integration**: Rhodium Standard Repository compliance checking

## Architecture

```
src/
├── main.rs           # CLI entry point
├── lib.rs            # Library exports
├── cli/              # Command handlers
│   ├── mod.rs        # CLI definitions (clap)
│   ├── init.rs       # `conflow init`
│   ├── analyze.rs    # `conflow analyze`
│   ├── run.rs        # `conflow run`
│   ├── validate.rs   # `conflow validate`
│   ├── watch.rs      # `conflow watch`
│   ├── graph.rs      # `conflow graph`
│   ├── cache.rs      # `conflow cache`
│   └── rsr.rs        # `conflow rsr`
├── pipeline/         # Pipeline orchestration
│   ├── definition.rs # Pipeline, Stage, Tool types
│   ├── dag.rs        # Dependency graph
│   ├── executor.rs   # Pipeline execution
│   └── validation.rs # Pipeline validation
├── executors/        # Tool executors
│   ├── cue.rs        # CUE executor
│   ├── nickel.rs     # Nickel executor
│   └── shell.rs      # Shell executor
├── cache/            # Caching system
│   ├── filesystem.rs # File-based cache
│   └── hash.rs       # Content hashing (BLAKE3)
├── analyzer/         # Config analysis
│   ├── complexity.rs # Complexity metrics
│   ├── config_detector.rs # Format detection
│   └── recommender.rs # Tool recommendations
├── rsr/              # RSR integration
│   ├── compliance.rs # Compliance checking
│   ├── requirements.rs # RSR requirements
│   ├── schemas.rs    # Schema registry
│   ├── hooks.rs      # External integration
│   ├── remediation.rs # Auto-fix
│   ├── badges.rs     # Badge generation
│   ├── diff.rs       # Compliance diffs
│   ├── config.rs     # .rsr.yaml loading
│   └── templates.rs  # Template generation
├── errors/           # Error handling
│   ├── mod.rs        # Error types (miette)
│   └── educational.rs # Helpful error messages
└── utils/            # Utilities
    ├── colors.rs     # Terminal colors
    └── spinner.rs    # Progress indicators
```

## Key Files

### `.conflow.yaml` Format

```yaml
version: "1"
name: pipeline-name

stages:
  - name: stage-name
    tool:
      type: cue | nickel | shell
      command: vet | export | eval | <shell-command>
      # Tool-specific options...
    input: <glob-pattern> | from_stage: <stage-name>
    output: <path>
    depends_on: [<stage-names>]
    description: Optional description

cache:
  enabled: true
  directory: .conflow-cache
```

### Important Types

```rust
// Pipeline definition (src/pipeline/definition.rs)
pub struct Pipeline {
    pub version: String,
    pub name: String,
    pub stages: Vec<Stage>,
    pub cache: Option<CacheConfig>,
}

// Stage definition
pub struct Stage {
    pub name: String,
    pub tool: Tool,
    pub input: Input,
    pub output: Option<Output>,
    pub depends_on: Vec<String>,
    pub description: Option<String>,
}

// Tool variants
pub enum Tool {
    Cue { command: CueCommand, schemas: Vec<PathBuf>, ... },
    Nickel { command: NickelCommand, format: OutputFormat, ... },
    Shell { command: String, shell: Option<String> },
}
```

## Development Guidelines

### Building

```bash
cargo build           # Debug build
cargo build --release # Release build
cargo test            # Run tests
cargo clippy          # Lint
cargo fmt             # Format
```

### Testing

- Unit tests: `cargo test`
- Integration tests: `cargo test --test '*'`
- Specific test: `cargo test test_name`

### Adding a New Executor

1. Create `src/executors/new_tool.rs`
2. Implement `Executor` trait
3. Add to `src/executors/mod.rs`
4. Add `Tool::NewTool` variant in `src/pipeline/definition.rs`
5. Handle in executor dispatch

### Adding a New CLI Command

1. Add variant to `Commands` enum in `src/cli/mod.rs`
2. Create `src/cli/command.rs` with `run()` function
3. Add dispatch in `src/main.rs`

## Code Style

- Use `cargo fmt` for formatting
- Add SPDX headers to all source files
- Document public APIs
- Handle errors explicitly (no `.unwrap()` in library code)
- Prefer `miette` for user-facing errors

## Common Tasks

### Running a pipeline
```rust
use conflow::pipeline::{Pipeline, PipelineExecutor, ExecutionOptions};

let pipeline = Pipeline::from_file(".conflow.yaml")?;
let executor = PipelineExecutor::new(pipeline);
let results = executor.run(ExecutionOptions::default()).await?;
```

### Checking RSR compliance
```rust
use conflow::rsr::ComplianceChecker;

let checker = ComplianceChecker::new();
let report = checker.check(project_root)?;
println!("Level: {:?}, Score: {:.0}%", report.level, report.score * 100.0);
```

### Generating from template
```rust
use conflow::rsr::TemplateGenerator;

let generator = TemplateGenerator::new();
let result = generator.generate("kubernetes", target_dir, &variables)?;
```

## RSR Compliance

This project aims for RSR Silver compliance:

- [x] Nix flake for reproducible builds
- [x] Justfile for task running
- [x] Dual MIT/Apache-2.0 license
- [x] Comprehensive documentation
- [x] TPCF contribution framework
- [x] Security policy
- [x] Code of Conduct

## Troubleshooting

### Common Issues

1. **CUE/Nickel not found**: Ensure they're in PATH or use Nix
2. **Cache issues**: Run `conflow cache clear`
3. **Pipeline validation errors**: Check `conflow validate`

### Debug Logging

```bash
RUST_LOG=conflow=debug conflow run
```

## Links

- Repository: https://gitlab.com/hyperpolymath/conflow
- RSR Standards: https://gitlab.com/hyperpolymath/rhodium-standard-repositories
- CUE: https://cuelang.org
- Nickel: https://nickel-lang.org

---

*This file follows RSR standards for AI assistant guidance.*
