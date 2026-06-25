<!--
SPDX-License-Identifier: CC-BY-SA-4.0
SPDX-FileCopyrightText: 2025-2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->

[![License: MPL-2.0](https://img.shields.io/badge/License-MPL_2.0--1.0-blue.svg)](https://github.com/hyperpolymath/palimpsest-license)
[![Palimpsest](https://img.shields.io/badge/Philosophy-Palimpsest-indigo.svg)](https://github.com/hyperpolymath/palimpsest-license)

Intelligently orchestrate CUE, Nickel, and configuration validation
workflows.

[![RSR Compliance](https://img.shields.io/badge/RSR-Silver-silver)](https://gitlab.com/hyperpolymath/rhodium-standard-repositories) image:[Rust
Version](https://img.shields.io/badge/Rust-1.75+-orange)

# Why conflow?

**Problem:** You have configuration files and you’re not sure whether to
use CUE, Nickel, or both.

**Solution:** conflow analyzes your configs, recommends the right tool,
and orchestrates the entire pipeline.

```bash
# Instead of:
nickel export config.ncl > temp.json
cue vet schema.cue temp.json
cue export schema.cue --out yaml > deploy.yaml
rm temp.json

# Just:
conflow run
```

# Features

- **Intelligent analysis** - Recommends CUE vs Nickel based on
  complexity

- **Pipeline orchestration** - Chain tools with dependency management

- **Smart caching** - Only re-run what changed

- **Educational** - Learn why certain tools fit certain problems

- **Type-safe** - Catch errors before deployment

- **RSR Integration** - Full Rhodium Standard Repository compliance
  checking

# Quick Start

```bash
# Install
cargo install conflow

# Initialize
conflow init my-project

# Analyze existing configs
conflow analyze config.yaml

# Run pipeline
conflow run
```

# Example Pipeline

```yaml
# .conflow.yaml
version: "1"
name: "k8s-deployment"

stages:
  - name: "generate"
    tool:
      type: nickel
      command: export
      file: config.ncl
    output: generated/config.json

  - name: "validate"
    tool:
      type: cue
      command: vet
      schemas: [schemas/k8s.cue]
    input:
      from_stage: generate
    depends_on: [generate]

  - name: "export"
    tool:
      type: cue
      command: export
      out_format: yaml
    input:
      from_stage: generate
    depends_on: [validate]
    output: deploy/k8s.yaml
```

```bash
$ conflow run
✓ generate (0.08s)
✓ validate (0.05s)
✓ export (0.03s)

Pipeline completed in 0.16s
```

# When to Use What?

## Use CUE when:

- ✅ Validating configuration

- ✅ Expressing constraints

- ✅ Merging configurations

- ✅ Simple transformations

## Use Nickel when:

- ✅ Generating configurations

- ✅ Complex logic needed

- ✅ Functions and abstraction

- ✅ DRY configuration

## Use Both when:

- ✅ Nickel generates → CUE validates

- ✅ Complex generation + strict validation

# Commands

| Command                                  | Description           |
|------------------------------------------|-----------------------|
| `conflow` `init` `[--template` `<name>]` | Initialize project    |
| `conflow` `analyze` `<files>`            | Analyze config files  |
| `conflow` `run` `[--stage` `<name>]`     | Execute pipeline      |
| `conflow` `watch`                        | Watch mode            |
| `conflow` `validate`                     | Validate pipeline     |
| `conflow` `graph` `[--format` `<fmt>]`   | Show pipeline graph   |
| `conflow` `cache` `stats`                | Cache statistics      |
| `conflow` `cache` `clear`                | Clear cache           |
| `conflow` `rsr` `check`                  | Check RSR compliance  |
| `conflow` `rsr` `requirements`           | List RSR requirements |

# Templates

```bash
conflow init --template cue-validation     # Simple CUE validation
conflow init --template nickel-generation  # Nickel config generation
conflow init --template full-pipeline      # Generate → validate → export
conflow init --template kubernetes         # Kubernetes manifests
conflow init --template multi-env          # Multi-environment configs
```

# RSR Compliance

conflow includes full RSR (Rhodium Standard Repository) integration:

- **Compliance checking** - Validate against RSR requirements

- **Auto-remediation** - Automatically fix common issues

- **Badge generation** - Generate compliance badges for CI

- **Diff reports** - Track compliance changes over time

```bash
# Check compliance
conflow rsr check

# Auto-fix issues
conflow rsr check --fix

# Generate badge
conflow rsr check --badge badge.svg
```

# Development

```bash
# Using Nix (recommended)
nix develop

# Using just
just build      # Build
just test       # Run tests
just check      # Run all checks
just install    # Install locally
```

# Documentation

- <a href="CLAUDE.md" class="md">CLAUDE</a> - AI assistant guidance

- <a href="CONTRIBUTING.md" class="md">CONTRIBUTING</a> - Contribution
  guidelines

- <a href="SECURITY.md" class="md">SECURITY</a> - Security policy

- <a href="GOVERNANCE.md" class="md">GOVERNANCE</a> - Project governance

- <a href="CODE_OF_CONDUCT.md" class="md">CODE_OF_CONDUCT</a> - Code of
  conduct

# RSR Standards

This project follows [Rhodium Standard
Repository](https://gitlab.com/hyperpolymath/rhodium-standard-repositories)
guidelines:

- ✅ Memory-safe language (Rust)

- ✅ Offline-first design

- ✅ Reproducible builds (Nix)

- ✅ Comprehensive documentation

- ✅ SPDX license headers

- ✅ Security policy

- ✅ TPCF contribution framework

# License

This project is dual-licensed under:

- MPL-2.0-1.0 License

- Apache License, Version 2.0

See <a href="LICENSE.txt" class="txt">LICENSE</a> for details.

# Contributing

Contributions are welcome! Please read our [Contributing
Guide](CONTRIBUTING.md) first.

# Links

- **Repository:** <https://gitlab.com/hyperpolymath/conflow>

- **Issues:** <https://gitlab.com/hyperpolymath/conflow/-/issues>

- **CUE:** <https://cuelang.org>

- **Nickel:** <https://nickel-lang.org>

- **RSR:**
  <https://gitlab.com/hyperpolymath/rhodium-standard-repositories>

# Architecture

See <a href="TOPOLOGY.md" class="md">TOPOLOGY</a> for a visual
architecture map and completion dashboard.
