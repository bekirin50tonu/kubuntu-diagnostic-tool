use crate::domain::{CheckResult, DiagnosticCheck, DiagnosticError, Severity};
use crate::infrastructure::system_info::MemoryInfoProvider;

const WARNING_PERCENT: f32 = 85.0;
const CRITICAL_PERCENT: f32 = 95.0;

/// Reports RAM (and swap, as extra context) usage.
pub struct MemoryCheck<P: MemoryInfoProvider> {
    provider: P,
}

impl<P: MemoryInfoProvider> MemoryCheck<P> {
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

impl<P: MemoryInfoProvider> DiagnosticCheck for MemoryCheck<P> {
    fn name(&self) -> &str {
        "memory"
    }

    fn evaluate(&self) -> Result<CheckResult, DiagnosticError> {
        let usage = self.provider.memory_usage();
        let severity = Self::severity_for(usage.used_percent);
        let message = format!("RAM usage is {:.1}%", usage.used_percent);

        Ok(CheckResult::new(self.name(), severity, message)
            .with_details(format!("swap usage is {:.1}%", usage.swap_used_percent)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::system_info::MemoryUsage;

    struct FakeMemoryProvider(MemoryUsage);
    impl MemoryInfoProvider for FakeMemoryProvider {
        fn memory_usage(&self) -> MemoryUsage {
            self.0
        }
    }

    #[test]
    fn reports_ok_under_threshold() {
        let check = MemoryCheck::new(FakeMemoryProvider(MemoryUsage {
            used_percent: 50.0,
            swap_used_percent: 0.0,
        }));
        assert_eq!(check.evaluate().unwrap().severity, Severity::Ok);
    }

    #[test]
    fn reports_warning_at_threshold() {
        let check = MemoryCheck::new(FakeMemoryProvider(MemoryUsage {
            used_percent: 88.0,
            swap_used_percent: 10.0,
        }));
        assert_eq!(check.evaluate().unwrap().severity, Severity::Warning);
    }

    #[test]
    fn reports_critical_when_nearly_exhausted() {
        let check = MemoryCheck::new(FakeMemoryProvider(MemoryUsage {
            used_percent: 99.0,
            swap_used_percent: 80.0,
        }));
        let result = check.evaluate().unwrap();
        assert_eq!(result.severity, Severity::Critical);
        assert!(result.details.unwrap().contains("80.0"));
    }
}
