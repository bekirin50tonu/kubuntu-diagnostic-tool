# Skill: Diagnostic Domain Model

The shared vocabulary every layer agrees on. Read this before naming
anything new.

## Core concepts

- **`Severity`** — `Ok`, `Warning`, `Error`, `Critical`. Ordered so a
  report can be sorted/filtered by "worst first".
- **`CheckResult`** — the output of one `DiagnosticCheck`: `check_name`,
  `severity`, `message`, and optional `details` (free-form extra context,
  e.g. the exact disk usage percentage).
- **`DiagnosticReport`** — a Composite of `CheckResult`s plus a
  `generated_at` timestamp and a convenience `overall_severity()` (the
  worst severity across all results).
- **`DiagnosticCheck`** (trait) — `fn name(&self) -> &str` and
  `fn evaluate(&self) -> Result<CheckResult, DiagnosticError>`. This is the
  Strategy interface every concrete check implements.
- **`Reporter`** (trait) — `fn render(&self, report: &DiagnosticReport) ->
  String`. Console and JSON reporters both implement this; the CLI picks
  one based on a flag.

## The checks this tool ships with (v1 scope)

| Check | What it inspects | Data source |
|---|---|---|
| `DiskCheck` | Free space per mounted filesystem, warns below a threshold | `sysinfo::Disks` |
| `MemoryCheck` | RAM + swap usage, warns on high pressure | `sysinfo::System` |
| `CpuCheck` | Load average / per-core usage | `sysinfo::System` |
| `NetworkCheck` | Whether the default route/DNS resolves | `sysinfo::Networks` + a lightweight TCP probe |
| `ServiceCheck` | Failed systemd units | `systemctl --failed --plain --no-legend` |

Each row is intentionally one struct, one file, one responsibility — this
table *is* the backlog for `infrastructure/checks/`.

## Thresholds
Keep thresholds as named constants at the top of each check file (e.g.
`const DISK_WARNING_PERCENT: f32 = 85.0;`), not magic numbers inline, so a
future "make thresholds configurable" feature only has to plumb a value
into an existing constant rather than hunt for literals.

## Naming conventions
- Check structs: `<Subject>Check` (`DiskCheck`, `MemoryCheck`).
- Reporter structs: `<Format>Reporter` (`ConsoleReporter`, `JsonReporter`).
- Every check's `name()` returns a short, stable, machine-friendly string
  (e.g. `"disk"`, `"memory"`) — this is what JSON output keys on, so it
  must never change once shipped without a version bump.
