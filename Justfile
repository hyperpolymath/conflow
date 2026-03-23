# conflow justfile
# Run `just` to see available recipes

# Default recipe - show help
default:
    @just --list

# Build the project
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Run tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Run clippy lints
lint:
    cargo clippy -- -D warnings

# Format code
fmt:
    cargo fmt

# Check formatting
fmt-check:
    cargo fmt -- --check

# Run all checks (lint, fmt, test)
check: fmt-check lint test

# Install locally
install:
    cargo install --path .

# Clean build artifacts
clean:
    cargo clean

# Run conflow with arguments
run *ARGS:
    cargo run -- {{ARGS}}

# Run conflow init
init *ARGS:
    cargo run -- init {{ARGS}}

# Run conflow analyze
analyze *ARGS:
    cargo run -- analyze {{ARGS}}

# Run conflow validate
validate *ARGS:
    cargo run -- validate {{ARGS}}

# Generate documentation
doc:
    cargo doc --no-deps --open

# Watch for changes and run tests
watch-test:
    cargo watch -x test

# Watch for changes and run
watch-run *ARGS:
    cargo watch -x "run -- {{ARGS}}"

# Run example: simple validation
example-simple:
    cd examples/simple && cargo run --manifest-path ../../Cargo.toml -- run

# Run example: nickel generation
example-generate:
    cd examples/generate && cargo run --manifest-path ../../Cargo.toml -- run

# Run example: full pipeline
example-full:
    cd examples/full-pipeline && cargo run --manifest-path ../../Cargo.toml -- run

# Run example: multi-environment
example-multi-env:
    cd examples/multi-env && cargo run --manifest-path ../../Cargo.toml -- run

# Validate all examples
validate-examples:
    @echo "Validating simple example..."
    cd examples/simple && cargo run --manifest-path ../../Cargo.toml -- validate
    @echo "Validating generate example..."
    cd examples/generate && cargo run --manifest-path ../../Cargo.toml -- validate
    @echo "Validating full-pipeline example..."
    cd examples/full-pipeline && cargo run --manifest-path ../../Cargo.toml -- validate
    @echo "Validating multi-env example..."
    cd examples/multi-env && cargo run --manifest-path ../../Cargo.toml -- validate

# Create a new release
release VERSION:
    @echo "Creating release {{VERSION}}..."
    git tag -a v{{VERSION}} -m "Release {{VERSION}}"
    cargo publish --dry-run

# Show dependency tree
deps:
    cargo tree

# Update dependencies
update:
    cargo update

# Audit dependencies for vulnerabilities
audit:
    cargo audit

# [AUTO-GENERATED] Multi-arch / RISC-V target
build-riscv:
	@echo "Building for RISC-V..."
	cross build --target riscv64gc-unknown-linux-gnu

# Run panic-attacker pre-commit scan
assail:
    @command -v panic-attack >/dev/null 2>&1 && panic-attack assail . || echo "panic-attack not found — install from https://github.com/hyperpolymath/panic-attacker"
