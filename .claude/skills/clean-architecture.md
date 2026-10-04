# Skill: Clean Architecture (applied to this codebase)

## The dependency rule
Source code dependencies only point inward:

```
presentation ──▶ application ──▶ domain ◀── infrastructure
```

`domain` is the center. It knows nothing about Rust crates like `sysinfo`,
`clap`, or `serde_json`, and nothing about the CLI or file system. It only
knows about diagnostic *concepts*: a check, a result, a severity, a report.

## Layer responsibilities

### `domain/`
- `entities.rs` — `CheckResult`, `Severity`, `DiagnosticReport` (a
  Composite of `CheckResult`s).
- `traits.rs` — `DiagnosticCheck` (Strategy interface) and `Reporter`
  (Observer/output interface). These are the *only* contracts the rest of
  the app is built around.
- `error.rs` — `DiagnosticError`, the one error enum every layer maps into.
- No `use` of `infrastructure::*` or `presentation::*`, ever. No direct
  file/network/process I/O.

### `application/`
- `diagnostic_service.rs` — `DiagnosticService`, which takes a
  `Vec<Box<dyn DiagnosticCheck>>` in its constructor (Dependency Injection)
  and produces a `DiagnosticReport` by running each one and collecting
  results. It does not know or care whether a check reads `/proc`, shells
  out to `systemctl`, or is a test fake.

### `infrastructure/`
- `checks/*.rs` — concrete `DiagnosticCheck` implementors: `DiskCheck`,
  `MemoryCheck`, `CpuCheck`, `NetworkCheck`, `ServiceCheck`. Each one is an
  **Adapter** between a real system data source (`sysinfo`, `systemctl`,
  `ping`) and the domain's `DiagnosticCheck` trait.
- `checks/registry.rs` — the **Factory**: `build_default_checks()` returns
  the `Vec<Box<dyn DiagnosticCheck>>` the composition root hands to
  `DiagnosticService`.
- `system_info.rs` — a thin wrapper around `sysinfo::System` so checks
  don't each manage their own `System` instance.

### `presentation/`
- `cli.rs` — `clap`-based argument parsing only. No business logic.
- `console_reporter.rs`, `json_reporter.rs` — `Reporter` implementors.
  Adding a third format (e.g. YAML) means adding a third file here, nothing
  else.

## Why this split matters for this project specifically
Kubuntu diagnostics inherently mix "what does a healthy system look like"
(a stable, testable concept) with "how do I ask systemd about failed
units" (a fragile, environment-dependent detail). Clean Architecture keeps
the fragile part swappable and the stable part unit-testable without a
real machine.
