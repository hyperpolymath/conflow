# Contributing to conflow

Thank you for your interest in contributing to conflow! This document provides
guidelines and information for contributors.

## Tri-Perimeter Contribution Framework (TPCF)

conflow uses a graduated trust model based on the RSR Tri-Perimeter Contribution
Framework:

### Perimeter 1: Core (Maintainers Only)

Changes to critical infrastructure require maintainer review and approval:

- Build system (`Cargo.toml`, `flake.nix`, `justfile`)
- CI/CD configuration (`.gitlab-ci.yml`)
- Security-sensitive code (`src/executors/shell.rs`)
- Release processes

### Perimeter 2: Expert (Trusted Contributors)

Experienced contributors may work on:

- New executor implementations
- Pipeline validation logic
- Cache algorithms
- RSR integration features
- Performance optimizations

**Requirements**: Previous accepted contributions, demonstrated expertise

### Perimeter 3: Community (Open to All)

Everyone is welcome to contribute:

- Documentation improvements
- Bug reports and fixes
- Test coverage
- Example pipelines
- Translations
- Issue triage

## Getting Started

### Prerequisites

- Rust 1.75+ (install via rustup)
- Nix (optional, for reproducible builds)
- CUE and Nickel (for integration tests)

### Development Setup

```bash
# Clone the repository
git clone https://gitlab.com/hyperpolymath/conflow.git
cd conflow

# Option 1: Use Nix (recommended)
nix develop

# Option 2: Manual setup
cargo build
cargo test
```

### Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture
```

## Contribution Process

### 1. Find or Create an Issue

- Check existing issues first
- Create a new issue for bugs or features
- Wait for maintainer feedback on large changes

### 2. Fork and Branch

```bash
git checkout -b feature/my-feature
# or
git checkout -b fix/issue-123
```

### 3. Make Changes

- Follow the code style (run `cargo fmt`)
- Add tests for new functionality
- Update documentation as needed
- Add SPDX headers to new files:

```rust
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors
```

### 4. Commit

Follow conventional commits:

```
feat: add new cue validation option
fix: handle empty pipeline gracefully
docs: update CLI usage examples
test: add cache invalidation tests
refactor: simplify stage execution
```

### 5. Submit Merge Request

- Fill out the MR template
- Link related issues
- Ensure CI passes
- Request review

## Code Style

### Rust Guidelines

- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Prefer explicit error handling over `.unwrap()`
- Document public APIs with doc comments
- Keep functions focused and small

### Documentation

- Use clear, concise language
- Include code examples where helpful
- Update README for user-facing changes
- Add inline comments for complex logic

## Testing Requirements

### Unit Tests

- All new functions should have tests
- Test edge cases and error conditions
- Use descriptive test names

### Integration Tests

- Test CLI commands in `tests/`
- Test with real CUE/Nickel files
- Verify cache behavior

### Test Coverage

We aim for >80% coverage on core modules.

## Review Process

### What Reviewers Look For

1. **Correctness**: Does the code work as intended?
2. **Tests**: Are there adequate tests?
3. **Documentation**: Is it documented?
4. **Style**: Does it follow conventions?
5. **Security**: Any security implications?

### Review Timeline

- Initial response: 2-3 business days
- Full review: 1 week for small changes
- Large changes may take longer

## Community

### Communication Channels

- GitLab Issues: Bug reports and feature requests
- Merge Requests: Code discussions
- Email: maintainers@conflow.dev

### Meetings

- No regular meetings currently
- Ad-hoc discussions as needed

## Recognition

Contributors are recognized in:

- `MAINTAINERS.md` for significant contributions
- Release notes for merged changes
- `humans.txt` for all contributors

## License

By contributing, you agree that your contributions will be licensed under the
same MIT OR Apache-2.0 dual license as the project.

## Questions?

Don't hesitate to ask! Open an issue or reach out to maintainers.

---

*This contributing guide follows RSR standards and TPCF principles.*
