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

# ═══════════════════════════════════════════════════════════════════════════════
# ONBOARDING & DIAGNOSTICS
# ═══════════════════════════════════════════════════════════════════════════════

# Check all required toolchain dependencies and report health
doctor:
    #!/usr/bin/env bash
    echo "═══════════════════════════════════════════════════"
    echo "  Conflow Doctor — Toolchain Health Check"
    echo "═══════════════════════════════════════════════════"
    echo ""
    PASS=0; FAIL=0; WARN=0
    check() {
        local name="$1" cmd="$2" min="$3"
        if command -v "$cmd" >/dev/null 2>&1; then
            VER=$("$cmd" --version 2>&1 | head -1)
            echo "  [OK]   $name — $VER"
            PASS=$((PASS + 1))
        else
            echo "  [FAIL] $name — not found (need $min+)"
            FAIL=$((FAIL + 1))
        fi
    }
    check "just"              just      "1.25" 
    check "git"               git       "2.40" 
    check "Rust (cargo)"      cargo     "1.80" 
    check "Zig"               zig       "0.13" 
# Optional tools
if command -v panic-attack >/dev/null 2>&1; then
    echo "  [OK]   panic-attack — available"
    PASS=$((PASS + 1))
else
    echo "  [WARN] panic-attack — not found (pre-commit scanner)"
    WARN=$((WARN + 1))
fi
    echo ""
    echo "  Result: $PASS passed, $FAIL failed, $WARN warnings"
    if [ "$FAIL" -gt 0 ]; then
        echo "  Run 'just heal' to attempt automatic repair."
        exit 1
    fi
    echo "  All required tools present."

# Attempt to automatically install missing tools
heal:
    #!/usr/bin/env bash
    echo "═══════════════════════════════════════════════════"
    echo "  Conflow Heal — Automatic Tool Installation"
    echo "═══════════════════════════════════════════════════"
    echo ""
if ! command -v cargo >/dev/null 2>&1; then
    echo "Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi
if ! command -v just >/dev/null 2>&1; then
    echo "Installing just..."
    cargo install just 2>/dev/null || echo "Install just from https://just.systems"
fi
    echo ""
    echo "Heal complete. Run 'just doctor' to verify."

# Guided tour of the project structure and key concepts
tour:
    #!/usr/bin/env bash
    echo "═══════════════════════════════════════════════════"
    echo "  Conflow — Guided Tour"
    echo "═══════════════════════════════════════════════════"
    echo ""
    echo 'Intelligently orchestrate CUE, Nickel, and configuration validation workflows.'
    echo ""
    echo "Key directories:"
    echo "  src/                      Source code" 
    echo "  ffi/                      Foreign function interface (Zig)" 
    echo "  src/abi/                  Idris2 ABI definitions" 
    echo "  docs/                     Documentation" 
    echo "  .github/workflows/        CI/CD workflows" 
    echo "  contractiles/             Must/Trust/Dust contracts" 
    echo "  .machine_readable/        Machine-readable metadata" 
    echo "  examples/                 Usage examples" 
    echo ""
    echo "Quick commands:"
    echo "  just doctor    Check toolchain health"
    echo "  just heal      Fix missing tools"
    echo "  just help-me   Common workflows"
    echo "  just default   List all recipes"
    echo ""
    echo "Read more: README.adoc, EXPLAINME.adoc"

# Show help for common workflows
help-me:
    #!/usr/bin/env bash
    echo "═══════════════════════════════════════════════════"
    echo "  Conflow — Common Workflows"
    echo "═══════════════════════════════════════════════════"
    echo ""
echo "FIRST TIME SETUP:"
echo "  just doctor           Check toolchain"
echo "  just heal             Fix missing tools"
echo "" 
    echo "DEVELOPMENT:" 
    echo "  cargo build           Build the project" 
    echo "  cargo test            Run tests" 
    echo "" 
echo "PRE-COMMIT:"
echo "  just assail           Run panic-attacker scan"
echo ""
echo "LEARN:"
echo "  just tour             Guided project tour"
echo "  just default          List all recipes" 
