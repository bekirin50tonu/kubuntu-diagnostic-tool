# Skill: SOLID Principles (applied to this codebase)

Use this file as a checklist whenever a trait, struct, or module is created
or changed.

## S — Single Responsibility Principle
A struct/module has exactly one reason to change.
- `DiagnosticCheck` implementors: reason to change = "the rule for this one
  check changed." They never format output and never orchestrate other
  checks.
- `Reporter` implementors: reason to change = "the output format changed."
  They never decide *what* to check.
- `DiagnosticService`: reason to change = "the orchestration policy
  changed" (ordering, parallelism, aggregation) — never the content of an
  individual check.

## O — Open/Closed Principle
The system is open for extension, closed for modification.
- Add a new check → add a new file implementing `DiagnosticCheck` + one line
  in `registry.rs`. `DiagnosticService` is never touched.
- Add a new output format → add a new `Reporter` implementor. The CLI only
  needs a new `match` arm to select it, nothing in the reporting logic
  itself changes.

## L — Liskov Substitution Principle
Any `DiagnosticCheck` must be usable wherever `DiagnosticCheck` is expected,
with no special-casing by the caller.
- No implementor should return an error type or panic in a way that a
  caller must specifically know about to handle correctly — all failures
  become a `CheckResult` with `Severity::Error`, or a `DiagnosticError`
  from the same enum every other check uses.
- No implementor should require being called in a specific order or with
  side effects another check depends on.

## I — Interface Segregation Principle
Traits stay small and role-specific.
- `DiagnosticCheck` exposes only `name()` and `evaluate()` — nothing a
  reporter needs, nothing a specific check family needs that others don't.
- If a future check family needs extra capability (e.g. "can this check be
  auto-fixed?"), define a separate trait (`Remediable`) that only the
  relevant checks implement, rather than growing `DiagnosticCheck` with an
  optional method every other implementor has to stub out.

## D — Dependency Inversion Principle
High-level modules (`application`) depend on abstractions (`domain`
traits), not on concrete infrastructure.
- `DiagnosticService` holds `Vec<Box<dyn DiagnosticCheck>>`, never a
  concrete `DiskCheck` or `MemoryCheck` type.
- The composition root (`main.rs`) is the only place concrete
  infrastructure types are constructed and wired to the trait objects the
  application layer consumes.
