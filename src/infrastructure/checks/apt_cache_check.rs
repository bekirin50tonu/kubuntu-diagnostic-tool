use crate::domain::{CheckResult, DiagnosticCheck, DiagnosticError, Severity};
use std::process::Command;

/// A provider trait for cleaning apt cache.
pub trait AptCacheProvider {
    fn clean_apt_cache(&self) -> Result<(), DiagnosticError>;
}

/// The real-world provider: shells out to `sudo apt-get clean`.
pub struct RealAptCacheProvider;

impl AptCacheProvider for RealAptCacheProvider {
    fn clean_apt_cache(&self) -> Result<(), DiagnosticError> {
        // Run sudo apt-get clean
        let _ = Command::new("sudo")
            .arg("apt-get")
            .arg("clean")
            .output()
            .map_err(|e| DiagnosticError::DataSourceUnavailable {
                check_name: "apt_cache".to_string(),
                reason: format!("failed to run sudo apt-get clean: {e}"),
            })?;

        Ok(())
    }
}

/// Reports the result of running apt cache cleaning.
pub struct AptCacheCheck<P: AptCacheProvider> {
    provider: P,
}

impl<P: AptCacheProvider> AptCacheCheck<P> {
    pub fn new(provider: P) -> Self {
        Self { provider }
    }
}

impl<P: AptCacheProvider> DiagnosticCheck for AptCacheCheck<P> {
    fn name(&self) -> &str {
        "apt_cache"
    }

    fn evaluate(&self) -> Result<CheckResult, DiagnosticError> {
        self.provider.clean_apt_cache()?;
        Ok(CheckResult::new(
            self.name(),
            Severity::Ok,
            "apt cache cleaned successfully",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeAptCacheProvider(Result<(), DiagnosticError>);
    impl AptCacheProvider for FakeAptCacheProvider {
        fn clean_apt_cache(&self) -> Result<(), DiagnosticError> {
            self.0.clone()
        }
    }

    #[test]
    fn reports_ok_when_apt_cache_clean_succeeds() {
        let check = AptCacheCheck::new(FakeAptCacheProvider(Ok(())));
        let result = check.evaluate().unwrap();
        assert_eq!(result.severity, Severity::Ok);
        assert_eq!(result.check_name, "apt_cache");
        assert!(result.message.contains("apt cache cleaned successfully"));
    }

    #[test]
    fn propagates_provider_errors() {
        let check = AptCacheCheck::new(FakeAptCacheProvider(Err(
            DiagnosticError::DataSourceUnavailable {
                check_name: "apt_cache".to_string(),
                reason: "sudo not found".to_string(),
            },
        )));
        assert!(check.evaluate().is_err());
    }
}