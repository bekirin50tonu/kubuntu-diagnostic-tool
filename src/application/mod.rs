//! The application layer: orchestrates domain abstractions into a use case.
//!
//! `DiagnosticService` depends only on `domain::DiagnosticCheck` — never on
//! a concrete check type — so it has zero knowledge of `sysinfo`,
//! `systemctl`, or anything else infrastructure-specific.

pub mod diagnostic_service;

pub use diagnostic_service::DiagnosticService;
