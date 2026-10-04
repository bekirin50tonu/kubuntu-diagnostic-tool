//! The domain layer: pure business rules for what a "diagnostic" is.
//!
//! Nothing in this module (or its children) may depend on
//! `crate::infrastructure` or `crate::presentation`. It also avoids I/O of
//! any kind. This is what keeps `DiagnosticCheck` and `Reporter`
//! implementors trivially unit-testable in isolation.

pub mod entities;
pub mod error;
pub mod traits;

pub use entities::{CheckResult, DiagnosticReport, Severity};
pub use error::DiagnosticError;
pub use traits::{Clock, DiagnosticCheck, Reporter};
