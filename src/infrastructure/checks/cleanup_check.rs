use crate::domain::{CheckResult, DiagnosticCheck, DiagnosticError, Severity};
use std::process::Command;

/// A small port around cleaning apt cache and journal logs.
pub trait CleanupProvider {
    fn cleanup_apt_cache_and_journal(&self) -> Result<(), DiagnosticError>;
}

/// The real-world provider: shells out to `sudo apt-get clean` and `sudo journalctl --vacuum-size=50M`.
pub struct RealCleanupProvider;

impl CleanupProvider for RealCleanupProvider {
    fn cleanup_apt_cache_and_journal(&self) -> Result<(), DiagnosticError> {
        // Run sudo apt-get clean
        let _ = Command::new("sudo")
            .arg("apt-get")
            .arg("clean")
            .output()
            .map_err(|e| DiagnosticError::DataSourceUnavailable {
                check_name: "cleanup".to_string(),
                reason: format!("failed to run sudo apt-get clean: {e}"),
            })?;

        // Run sudo journalctl --vacuum-size=50M
        let _ = Command::new("sudo")
            .arg("journalctl")
            .arg("--vacuum-size=50M")
            .output()
            .map_err(|e| DiagnosticError::DataSourceUnavailable {
                check_name: "cleanup".to_string(),
                reason: format!("failed to run sudo journalctl --vacuum-size=50M: {e}"),
            })?;

        Ok(())
    }
}

/// Reports the result of running apt cache and journal cleanup.
pub struct CleanupCheck<P: CleanupProvider> {
    provider: P,
}

impl<P: CleanupProvider> CleanupCheck<P> {
    pub fn new(provider: P) -> Self {
        Self { provider }
    }
}

impl<P: CleanupProvider> DiagnosticCheck for CleanupCheck<P> {
    fn name(&self) -> &str {
        "cleanup"
    }

    fn evaluate(&self) -> Result<CheckResult, DiagnosticError> {
        self.provider.cleanup_apt_cache_and_journal()?;
        Ok(CheckResult::new(
            self.name(),
            Severity::Ok,
            "apt cache and journal cleanup succeeded",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeCleanupProvider(Result<(), DiagnosticError>);
    impl CleanupProvider for FakeCleanupProvider {
        fn cleanup_apt_cache_and_journal(&self) -> Result<(), DiagnosticError> {
            self.0.clone()
        }
    }

    #[test]
    fn reports_ok_when_cleanup_succeeds() {
        let check = CleanupCheck::new(FakeCleanupProvider(Ok(())));
        let result = check.evaluate().unwrap();
        assert_eq!(result.severity, Severity::Ok);
        assert_eq!(result.check_name, "cleanup");
        assert!(result
            .message
            .contains("apt cache and journal cleanup succeeded"));
    }

    #[test]
    fn propagates_provider_errors() {
        let check = CleanupCheck::new(FakeCleanupProvider(Err(
            DiagnosticError::DataSourceUnavailable {
                check_name: "cleanup".to_string(),
                reason: "sudo not found".to_string(),
            },
        )));
        assert!(check.evaluate().is_err());
    }
}