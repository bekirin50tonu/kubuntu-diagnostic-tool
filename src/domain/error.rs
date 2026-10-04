use std::fmt;

/// The one error type every layer maps its failures into.
///
/// Keeping a single, small error enum (instead of `Box<dyn Error>`
/// everywhere) lets `application` and `presentation` match on failure
/// *kind* without knowing which crate or subsystem actually failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticError {
    /// The data source needed for a check was unavailable (e.g. a command
    /// isn't installed, or a `/proc` entry is missing).
    DataSourceUnavailable { check_name: String, reason: String },
    /// The data source responded, but in a shape the check couldn't parse.
    UnexpectedOutput { check_name: String, reason: String },
    /// Rendering a report in a given format failed.
    RenderFailure { format: String, reason: String },
}

impl fmt::Display for DiagnosticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DataSourceUnavailable { check_name, reason } => {
                write!(f, "check '{check_name}' has no data source: {reason}")
            }
            Self::UnexpectedOutput { check_name, reason } => {
                write!(f, "check '{check_name}' got unexpected output: {reason}")
            }
            Self::RenderFailure { format, reason } => {
                write!(f, "failed to render report as '{format}': {reason}")
            }
        }
    }
}

impl std::error::Error for DiagnosticError {}
