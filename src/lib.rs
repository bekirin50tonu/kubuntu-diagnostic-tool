//! Library crate exposing the layered architecture so both the binary
//! (`main.rs`) and integration tests can build on the same composition
//! logic without duplicating it.

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;

use application::DiagnosticService;
use domain::DiagnosticReport;
use infrastructure::checks::build_default_checks;
use infrastructure::SystemClock;

/// Convenience entry point used by the binary and by integration tests:
/// wires the default checks and the real system clock, then runs them.
pub fn run_default_diagnostics() -> DiagnosticReport {
    let service = DiagnosticService::new(build_default_checks(), Box::new(SystemClock));
    service.run_all()
}
