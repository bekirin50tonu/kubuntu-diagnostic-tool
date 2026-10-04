# kubuntu-diagnostic-tool

A Kubuntu system diagnostic CLI written in Rust, built as a reference
implementation of Clean Architecture + SOLID for use with Claude Code.

Checks: disk usage, RAM/swap usage, CPU usage, outbound network
connectivity, and failed systemd units.

## For Claude Code

Read `CLAUDE.md` first — it's the project charter (architecture rules,
non-negotiables, workflow). `AGENT.md` explains which persona under
`.claude/agents/` to use for a given task, and `.claude/skills/` holds the
reference material (SOLID, Clean Architecture, Rust idioms, the diagnostic
domain model) those agents consult.

## Build & run

```bash
cargo build --release
./target/release/kubuntu-diagnostic-tool
./target/release/kubuntu-diagnostic-tool --format json
./target/release/kubuntu-diagnostic-tool --fail-on critical
```

## Test

```bash
cargo test
```

Verified on this build: `cargo build`, `cargo test` (26/26 passing), and a
manual run of both output formats — all clean, no compiler warnings.

## Architecture

```
src/
├── domain/           # Pure business rules — no I/O, no external deps
├── application/       # DiagnosticService — orchestrates checks
├── infrastructure/     # sysinfo/systemctl/TCP adapters, the check factory
└── presentation/       # CLI (clap) + console/JSON reporters
```

Dependency rule: `presentation → application → domain`,
`infrastructure → domain`. Adding a new check means adding one file under
`src/infrastructure/checks/` and one line in `registry.rs` — nothing else
changes (Open/Closed Principle).

## Notes on this build environment

Developed and tested against rustc/cargo 1.75. `sysinfo` is pinned to
`0.30.13` with default features disabled (drops an MSRV-incompatible
`rayon` transitive dependency); `clap` is pinned to `4.4.18` for the same
reason. On a newer toolchain these pins can be relaxed if desired.
`cargo clippy` / `cargo fmt` were not available in the verification
environment (no rustup/clippy component installed) — code was written
manually to the standards in `.claude/skills/rust-idioms.md`; run clippy
locally before merging real changes.
