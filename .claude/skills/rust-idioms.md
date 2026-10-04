# Skill: Rust Idioms & Optimization Checklist

Apply this before considering any Rust change finished.

## Error handling
- Fallible functions return `Result<T, DiagnosticError>`. Never `unwrap()`,
  `expect()`, or `panic!()` outside `#[cfg(test)]`.
- Use `?` for propagation. Implement `From<SourceError> for
  DiagnosticError` instead of `.map_err(|e| ...)` scattered everywhere.
- A single check failing (e.g. a command not found) must degrade to a
  `CheckResult` with `Severity::Error` and a message — it must never abort
  the whole run.

## Ownership & borrowing
- Function parameters take `&str` / `&[T]` / `&T` unless the function needs
  to own or outlive the caller's data.
- Avoid `.clone()` as a first resort. If a clone is genuinely needed
  (e.g. moving data into a spawned task), leave a short comment saying why.

## Iterators over manual loops
- Prefer `.iter().map(...).collect()`, `.filter(...)`, `.fold(...)`,
  `.any()`, `.all()` over hand-written `for` loops with a mutable
  accumulator, when it doesn't hurt readability.
- `DiagnosticService::run_all` is the canonical example: iterate the checks,
  map each to a `CheckResult`, collect into the report — no manual index
  bookkeeping.

## Trait objects vs generics
- Use `Box<dyn Trait>` only at genuine plugin boundaries where a
  heterogeneous collection is required (`Vec<Box<dyn DiagnosticCheck>>`).
- Inside a single check's implementation, prefer concrete types and
  generics — no unnecessary dynamic dispatch.

## Types over primitives
- Wrap raw numbers with meaning in newtypes or enums where confusion is
  possible (`Severity::{Ok, Warning, Error, Critical}` instead of a bare
  `u8`; a `Percentage(f32)` newtype instead of a bare `f32` for disk usage
  if it's compared against thresholds in multiple places).

## Formatting & linting
- `cargo fmt` before every commit.
- `cargo clippy --all-targets -- -D warnings` must be clean. Do not silence
  a lint with `#[allow(...)]` without a comment explaining why it's a false
  positive here.

## Documentation
- Every public trait, struct, and function gets a `///` doc comment. State
  *why* it exists / *when* to use it, not just a restatement of the name.
