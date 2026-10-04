use crate::domain::{CheckResult, DiagnosticCheck, DiagnosticError, Severity};
use std::process::Command;

/// A small port around "which systemd units are failed", so `ServiceCheck`
/// never has to know how to invoke or parse `systemctl` output.
pub trait FailedUnitsProvider {
    fn failed_units(&self) -> Result<Vec<String>, DiagnosticError>;
}

/// The real-world provider: shells out to `systemctl --failed`.
pub struct SystemctlFailedUnitsProvider;

impl FailedUnitsProvider for SystemctlFailedUnitsProvider {
    fn failed_units(&self) -> Result<Vec<String>, DiagnosticError> {
        let output = Command::new("systemctl")
            .args(["--failed", "--plain", "--no-legend"])
            .output()
            .map_err(|error| DiagnosticError::DataSourceUnavailable {
                check_name: "systemd_services".to_string(),
                reason: format!("could not run systemctl: {error}"),
            })?;

        if !output.status.success() {
            return Err(DiagnosticError::UnexpectedOutput {
                check_name: "systemd_services".to_string(),
                reason: format!("systemctl exited with status {}", output.status),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let units = stdout
            .lines()
            .filter_map(|line| line.split_whitespace().next())
            .map(str::to_string)
            .collect();
        Ok(units)
    }
}

/// Reports any systemd units currently in a failed state.
pub struct ServiceCheck<P: FailedUnitsProvider> {
    provider: P,
}

impl<P: FailedUnitsProvider> ServiceCheck<P> {
    pub fn new(provider: P) -> Self {
        Self { provider }
    }
}

impl<P: FailedUnitsProvider> DiagnosticCheck for ServiceCheck<P> {
    fn name(&self) -> &str {
        "systemd_services"
    }

    fn evaluate(&self) -> Result<CheckResult, DiagnosticError> {
        let failed_units = self.provider.failed_units()?;

        let result = if failed_units.is_empty() {
            CheckResult::new(self.name(), Severity::Ok, "no failed systemd units")
        } else {
            CheckResult::new(
                self.name(),
                Severity::Error,
                format!(
                    "{} unit(s) failed: {}",
                    failed_units.len(),
                    failed_units.join(", ")
                ),
            )
        };
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeUnitsProvider(Result<Vec<String>, DiagnosticError>);
    impl FailedUnitsProvider for FakeUnitsProvider {
        fn failed_units(&self) -> Result<Vec<String>, DiagnosticError> {
            self.0.clone()
        }
    }

    #[test]
    fn reports_ok_when_nothing_failed() {
        let check = ServiceCheck::new(FakeUnitsProvider(Ok(vec![])));
        assert_eq!(check.evaluate().unwrap().severity, Severity::Ok);
    }

    #[test]
    fn reports_error_listing_failed_units() {
        let check = ServiceCheck::new(FakeUnitsProvider(Ok(vec!["nginx.service".to_string()])));
        let result = check.evaluate().unwrap();
        assert_eq!(result.severity, Severity::Error);
        assert!(result.message.contains("nginx.service"));
    }

    #[test]
    fn propagates_provider_errors() {
        let check = ServiceCheck::new(FakeUnitsProvider(Err(
            DiagnosticError::DataSourceUnavailable {
                check_name: "systemd_services".to_string(),
                reason: "systemctl not found".to_string(),
            },
        )));
        assert!(check.evaluate().is_err());
    }
}
