use crate::domain::{CheckResult, DiagnosticCheck, DiagnosticError, Severity};
use crate::infrastructure::system_info::DiskInfoProvider;

const WARNING_PERCENT: f32 = 80.0;
const CRITICAL_PERCENT: f32 = 95.0;

/// Reports the fullest mounted filesystem's usage.
///
/// Depends only on `DiskInfoProvider` (Dependency Inversion), so tests can
/// substitute a fake list of disks instead of reading the real filesystem.
pub struct DiskCheck<P: DiskInfoProvider> {
    provider: P,
}

impl<P: DiskInfoProvider> DiskCheck<P> {
    pub fn new(provider: P) -> Self {
        Self { provider }
    }

    fn severity_for(used_percent: f32) -> Severity {
        if used_percent >= CRITICAL_PERCENT {
            Severity::Critical
        } else if used_percent >= WARNING_PERCENT {
            Severity::Warning
        } else {
            Severity::Ok
        }
    }
}

impl<P: DiskInfoProvider> DiagnosticCheck for DiskCheck<P> {
    fn name(&self) -> &str {
        "disk"
    }

    fn evaluate(&self) -> Result<CheckResult, DiagnosticError> {
        let usages = self.provider.disk_usages();

        let Some(fullest) = usages
            .iter()
            .max_by(|a, b| a.used_percent.total_cmp(&b.used_percent))
        else {
            return Err(DiagnosticError::DataSourceUnavailable {
                check_name: self.name().to_string(),
                reason: "no mounted filesystems reported".to_string(),
            });
        };

        let severity = Self::severity_for(fullest.used_percent);
        let message = format!(
            "'{}' is {:.1}% full",
            fullest.mount_point, fullest.used_percent
        );

        Ok(CheckResult::new(self.name(), severity, message)
            .with_details(format!("{} filesystem(s) checked", usages.len())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::system_info::DiskUsage;

    struct FakeDiskProvider(Vec<DiskUsage>);
    impl DiskInfoProvider for FakeDiskProvider {
        fn disk_usages(&self) -> Vec<DiskUsage> {
            self.0.clone()
        }
    }

    fn disk(mount_point: &str, used_percent: f32) -> DiskUsage {
        DiskUsage {
            mount_point: mount_point.to_string(),
            used_percent,
        }
    }

    #[test]
    fn reports_ok_when_usage_is_low() {
        let check = DiskCheck::new(FakeDiskProvider(vec![disk("/", 40.0)]));
        let result = check.evaluate().unwrap();
        assert_eq!(result.severity, Severity::Ok);
    }

    #[test]
    fn reports_warning_at_threshold() {
        let check = DiskCheck::new(FakeDiskProvider(vec![disk("/", 82.0)]));
        let result = check.evaluate().unwrap();
        assert_eq!(result.severity, Severity::Warning);
    }

    #[test]
    fn reports_critical_when_nearly_full() {
        let check = DiskCheck::new(FakeDiskProvider(vec![disk("/", 97.5)]));
        let result = check.evaluate().unwrap();
        assert_eq!(result.severity, Severity::Critical);
        assert!(result.message.contains("97.5"));
    }

    #[test]
    fn picks_the_fullest_of_several_disks() {
        let check = DiskCheck::new(FakeDiskProvider(vec![
            disk("/", 30.0),
            disk("/home", 91.0),
            disk("/boot", 10.0),
        ]));
        let result = check.evaluate().unwrap();
        assert!(result.message.contains("/home"));
        assert_eq!(result.severity, Severity::Warning);
    }

    #[test]
    fn errors_when_no_disks_are_reported() {
        let check = DiskCheck::new(FakeDiskProvider(vec![]));
        assert!(check.evaluate().is_err());
    }
}
