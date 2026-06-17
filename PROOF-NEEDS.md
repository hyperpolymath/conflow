<!--
SPDX-License-Identifier: MPL-2.0
Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->
# Proof Requirements

## Current state
- `src/abi/Types.idr` (232 lines) — Configuration flow types
- `src/abi/Layout.idr` (177 lines) — Memory layout
- `src/abi/Foreign.idr` (217 lines) — FFI declarations
- No dangerous patterns in ABI layer
- Claims: "type-safe", "memory-safe" (Rust)

## What needs proving
- **Configuration DAG acyclicity**: Prove the configuration dependency graph is always a DAG (no circular dependencies that cause infinite loops)
- **Merge conflict resolution determinism**: Prove that configuration merges produce deterministic results regardless of evaluation order
- **Rollback safety**: Prove that configuration rollback restores the exact previous state (no partial rollback)
- **Schema validation completeness**: Prove that all configuration values passing validation conform to their declared schema (no type confusion)

## Recommended prover
- **Idris2** — Dependent types naturally express DAG properties and schema conformance; already used for ABI

## Priority
- **MEDIUM** — Configuration errors can cascade to downstream services. The type-safety claim should be backed by proofs, especially for merge determinism and rollback correctness.

## Template ABI Cleanup (2026-03-29)

Template ABI removed -- was creating false impression of formal verification.
The removed files (Types.idr, Layout.idr, Foreign.idr) contained only RSR template
scaffolding with unresolved {{PROJECT}}/{{AUTHOR}} placeholders and no domain-specific proofs.
