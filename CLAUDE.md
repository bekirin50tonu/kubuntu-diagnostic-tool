# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A **Kubuntu system diagnostic tool** written in **Rust**. It inspects disk, memory, CPU, network, systemd services and package state, then reports problems in a human-readable or machine-readable format. Built as a reference implementation of Clean Architecture + SOLID principles.

## Quick Start

```bash
# Build
cargo build --release

# Run (human-readable)
./target/release/kubuntu-diagnostic-tool

# Run (JSON output)
./target/release/kubuntu-diagnostic-tool --format json

# Run (fail CI on warnings)
./target/release/kubuntu-diagnostic-tool --fail-on warning

# Test
cargo test

# Lint & format
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## Architecture

Clean Architecture with strict layering. Dependencies point inward only:

```
presentation → application → domain ← infrastructure
```

| Layer | Responsibility | Key Files |
|-------|----------------|-----------|
| `domain/` | Pure business rules — no I/O, no external deps besides `serde` | `entities.rs`, `traits.rs`, `error.rs` |
| `application/` | Use-case orchestration (`DiagnosticService`) | `diagnostic_service.rs` |
| `infrastructure/` | Concrete adapters: sysinfo, systemctl, TCP probes, check factory | `checks/*.rs`, `registry.rs`, `system_info.rs`, `system_clock.rs` |
| `presentation/` | CLI parsing (clap) + reporters (console, JSON) | `cli.rs`, `console_reporter.rs`, `json_reporter.rs` |

**Dependency Inversion**: `DiagnosticService` depends on `domain::DiagnosticCheck` trait, not concrete checks. The factory in `infrastructure/checks/registry.rs` wires the real implementations.

## Adding a New Diagnostic Check

This is the most common task. Follow these steps exactly:

1. Create `src/infrastructure/checks/<name>_check.rs`
2. Implement `domain::traits::DiagnosticCheck` for a small struct (use provider traits for testability)
3. Register the struct in `src/infrastructure/checks/registry.rs::build_default_checks()`
4. Add a unit test that exercises `evaluate()` against a fake/mocked data source — **do not depend on real machine state in unit tests**

**Nothing else needs to change.** If you find yourself editing `application::DiagnosticService` to add a check, stop — that violates the Open/Closed Principle.

## Key Patterns in This Codebase

- **Strategy pattern**: `DiagnosticCheck` trait — each check is a strategy
- **Factory pattern**: `build_default_checks()` in `registry.rs` builds the check list
- **Adapter pattern**: Each check wraps a provider trait (`DiskInfoProvider`, `MemoryInfoProvider`, etc.) to isolate `sysinfo`/`systemctl` details
- **Observer pattern**: `Reporter` trait — console/JSON are implementations
- **Dependency Injection**: `DiagnosticService` takes `Vec<Box<dyn DiagnosticCheck>>` and `Box<dyn Clock>` in constructor

## Non-Negotiable Rules

1. **Clean Architecture layering** — `domain` never uses `infrastructure` or `presentation`
2. **SOLID** — every new type/function checked against `.claude/skills/solid-principles.md`
3. **No god objects/functions** — single responsibility per type/function
4. **Every check is a plugin** — implements `DiagnosticCheck`, registered in factory
5. **No `unwrap()`/`expect()` outside tests** — use `Result` + `DiagnosticError`
6. **English only** — code, comments, commit messages, doc comments
7. **Idiomatic Rust** — prefer borrowing, iterators, zero-cost abstractions
8. **Tests mandatory** for domain and application logic; infrastructure gets integration tests behind `#[cfg(target_os = "linux")]`

## Agent Routing (AGENT.md)

Use the appropriate agent for the task:
- **architect** — deciding where code belongs, module boundaries, reviewing Clean Architecture + SOLID compliance
- **rust-developer** — writing/modifying Rust implementation inside already-decided modules
- **code-reviewer** — reviewing diffs/PRs: SOLID compliance, clippy warnings, error handling, naming
- **test-engineer** — writing unit/integration tests, designing fakes/mocks for `DiagnosticCheck` and `Reporter`

## Skills Reference

Read these before working in the codebase:
- `.claude/skills/clean-architecture.md` — dependency rules, layer responsibilities
- `.claude/skills/diagnostic-domain.md` — core concepts, check inventory, thresholds, naming
- `.claude/skills/solid-principles.md` — SOLID applied to this project
- `.claude/skills/rust-idioms.md` — Rust style guide for this project

## Integration Tests

Located in `tests/integration_test.rs`. Gated to `#[cfg(target_os = "linux")]`. Run against real machine state but resilient to missing `systemctl`.

```bash
cargo test --test integration_test
```

## Version Pins

- `sysinfo = "=0.30.13"` (default-features = false — drops `rayon` transitive dep)
- `clap = "=4.4.18"` (pinned for MSRV compatibility)
- Can be relaxed on newer toolchains if needed