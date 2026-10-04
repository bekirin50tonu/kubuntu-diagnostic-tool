use crate::domain::entities::{CheckResult, DiagnosticReport};
use crate::domain::error::DiagnosticError;

/// The Strategy interface for a single diagnostic rule.
///
/// Every concrete check (disk, memory, CPU, network, systemd services...)
/// implements this trait and nothing else in the codebase needs to change
/// to add a new one (Open/Closed Principle). The trait is deliberately
/// small (Interface Segregation): a check only needs to say who it is and
/// how to evaluate itself.
pub trait DiagnosticCheck {
    /// A short, stable, machine-friendly identifier (e.g. `"disk"`). This
    /// is used as a key in machine-readable output, so it must not change
    /// once shipped without a version bump.
    fn name(&self) -> &str;

    /// Run the check and produce exactly one result.
    ///
    /// A check that cannot reach its data source returns a
    /// `DiagnosticError` rather than panicking — the caller decides how to
    /// surface that (see `DiagnosticService::run_all`).
    fn evaluate(&self) -> Result<CheckResult, DiagnosticError>;
}

/// A tiny port around "what time is it now", so `DiagnosticService` can be
/// unit-tested with a fixed timestamp instead of the real system clock.
pub trait Clock {
    fn now_rfc3339(&self) -> String;
}

/// The Observer/output interface: turn a finished report into text.
///
/// `ConsoleReporter` and `JsonReporter` both implement this. Adding a third
/// format means adding a third implementor — `DiagnosticService` and every
/// `DiagnosticCheck` remain untouched.
pub trait Reporter {
    fn render(&self, report: &DiagnosticReport) -> Result<String, DiagnosticError>;
}
