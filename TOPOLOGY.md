<!-- SPDX-License-Identifier: PMPL-1.0-or-later -->
<!-- TOPOLOGY.md — Project architecture map and completion dashboard -->
<!-- Last updated: 2026-02-19 -->

# conflow — Project Topology

## System Architecture

```
                        ┌─────────────────────────────────────────┐
                        │              OPERATOR / CLI             │
                        │        (conflow run / init / watch)     │
                        └───────────────────┬─────────────────────┘
                                            │
                                            ▼
                        ┌─────────────────────────────────────────┐
                        │           CONFLOW ORCHESTRATOR          │
                        │    (Rust, Pipeline Mgmt, Caching)       │
                        └──────────┬───────────────────┬──────────┘
                                   │                   │
                                   ▼                   ▼
                        ┌───────────────────────┐  ┌────────────────────────────────┐
                        │ CONFIG ANALYSIS       │  │ RSR COMPLIANCE ENGINE          │
                        │ - Tool Recommendation │  │ - Validation & Auto-fix        │
                        │ - Dependency Graph    │  │ - Badge Generation             │
                        └──────────┬────────────┘  └──────────┬─────────────────────┘
                                   │                          │
                                   └────────────┬─────────────┘
                                                ▼
                        ┌─────────────────────────────────────────┐
                        │           TOOLCHAIN INTERFACE           │
                        │  ┌───────────┐  ┌───────────┐  ┌───────┐│
                        │  │ Nickel    │  │ CUE       │  │ Shell ││
                        │  │ (Gen)     │  │ (Vet/Exp) │  │ (Hook)││
                        │  └─────┬─────┘  └─────┬─────┘  └───────┘│
                        └────────│──────────────│─────────────────┘
                                 │              │
                                 ▼              ▼
                        ┌─────────────────────────────────────────┐
                        │          CONFIGURATION ASSETS           │
                        │      (config.ncl, schema.cue, etc.)     │
                        └─────────────────────────────────────────┘

                        ┌─────────────────────────────────────────┐
                        │          REPO INFRASTRUCTURE            │
                        │  Justfile / Nix     .machine_readable/  │
                        │  ClusterFuzzLite    RSR Silver Tier     │
                        └─────────────────────────────────────────┘
```

## Completion Dashboard

```
COMPONENT                          STATUS              NOTES
─────────────────────────────────  ──────────────────  ─────────────────────────────────
CORE ORCHESTRATOR
  Pipeline Runner (Rust)            ██████████ 100%    Dependency stages stable
  Smart Caching                     ████████░░  80%    Hashing logic refining
  Config Analyzer                   ██████████ 100%    Nickel vs CUE heuristic verified
  CLI Interface (conflow)           ██████████ 100%    Full command set active

RSR INTEGRATION
  Compliance Engine                 ██████████ 100%    Silver tier rules verified
  Auto-remediation                  ██████░░░░  60%    Fix patterns expanding
  Badge Generation                  ██████████ 100%    SVG templates stable

REPO INFRASTRUCTURE
  Justfile / Nix                    ██████████ 100%    Reproducible build env
  .machine_readable/                ██████████ 100%    STATE.a2ml tracking
  Fuzz Testing                      ████████░░  80%    ClusterFuzzLite active

─────────────────────────────────────────────────────────────────────────────
OVERALL:                            █████████░  ~90%   v0.1.0 RSR Silver Compliant
```

## Key Dependencies

```
conflow.yaml ───► Dependency Graph ───► Nickel Export ───► CUE Vet
     │                 │                   │                │
     ▼                 ▼                   ▼                ▼
Cache Store ─────► Action Plan ───────► JSON Output ────► Deployment
```

## Update Protocol

This file is maintained by both humans and AI agents. When updating:

1. **After completing a component**: Change its bar and percentage
2. **After adding a component**: Add a new row in the appropriate section
3. **After architectural changes**: Update the ASCII diagram
4. **Date**: Update the `Last updated` comment at the top of this file

Progress bars use: `█` (filled) and `░` (empty), 10 characters wide.
Percentages: 0%, 10%, 20%, ... 100% (in 10% increments).
