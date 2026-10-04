use serde::Serialize;
use std::fmt;

/// How bad a single diagnostic finding is.
///
/// Ordered from least to most severe so a report can sort or filter by
/// "worst first" using the derived `Ord` implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Ok,
    Warning,
    Error,
    Critical,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Ok => "OK",
            Self::Warning => "WARNING",
            Self::Error => "ERROR",
            Self::Critical => "CRITICAL",
        };
        write!(f, "{label}")
    }
}

/// Action that can be performed to clean up the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum CleanupAction {
    /// Clean apt-get cache via `apt-get clean`.
    AptCacheClean,
    /// Vacuum journal logs to limit size via `journalctl --vacuum-size=50M`.
    JournalVacuum,
}

/// The outcome of running one `DiagnosticCheck`.
///
/// This is the only thing a check is allowed to produce — it never writes
/// to stdout, a file, or a socket itself. That responsibility belongs to a
/// `Reporter`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CheckResult {
    pub check_name: String,
    pub severity: Severity,
    pub message: String,
    /// Free-form extra context (e.g. "87.3% used"), optional by design so
    /// simple checks aren't forced to populate a field they don't need.
    pub details: Option<String>,
}

impl CheckResult {
    pub fn new(check_name: impl Into<String>, severity: Severity, message: impl Into<String>) -> Self {
        Self {
            check_name: check_name.into(),
            severity,
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

/// A Composite of every `CheckResult` produced in one diagnostic run.
#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticReport {
    pub generated_at: String,
    pub results: Vec<CheckResult>,
}

impl DiagnosticReport {
    pub fn new(generated_at: impl Into<String>, results: Vec<CheckResult>) -> Self {
        Self {
            generated_at: generated_at.into(),
            results,
        }
    }

    /// The worst severity across all results, or `Severity::Ok` for an
    /// empty report. Used by the CLI to decide the process exit code.
    pub fn overall_severity(&self) -> Severity {
        self.results
            .iter()
            .map(|result| result.severity)
            .max()
            .unwrap_or(Severity::Ok)
    }

    pub fn results_at_or_above(&self, minimum: Severity) -> impl Iterator<Item = &CheckResult> {
        self.results.iter().filter(move |result| result.severity >= minimum)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_ordering_puts_critical_above_ok() {
        assert!(Severity::Critical > Severity::Ok);
        assert!(Severity::Warning < Severity::Error);
    }

    #[test]
    fn overall_severity_of_empty_report_is_ok() {
        let report = DiagnosticReport::new("2026-01-01T00:00:00Z", vec![]);
        assert_eq!(report.overall_severity(), Severity::Ok);
    }

    #[test]
    fn overall_severity_is_the_worst_result() {
        let report = DiagnosticReport::new(
            "2026-01-01T00:00:00Z",
            vec![
                CheckResult::new("disk", Severity::Warning, "low space"),
                CheckResult::new("memory", Severity::Critical, "out of memory"),
                CheckResult::new("cpu", Severity::Ok, "nominal"),
            ],
        );
        assert_eq!(report.overall_severity(), Severity::Critical);
    }

    #[test]
    fn results_at_or_above_filters_correctly() {
        let report = DiagnosticReport::new(
            "2026-01-01T00:00:00Z",
            vec![
                CheckResult::new("disk", Severity::Warning, "low space"),
                CheckResult::new("cpu", Severity::Ok, "nominal"),
            ],
        );
        let filtered: Vec<_> = report.results_at_or_above(Severity::Warning).collect();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].check_name, "disk");
    }
}