use crate::domain::{CheckResult, DiagnosticCheck, DiagnosticError, Severity};
use crate::infrastructure::system_info::CpuInfoProvider;

const WARNING_PERCENT: f32 = 85.0;
const CRITICAL_PERCENT: f32 = 97.0;

/// Reports global CPU utilization.
pub struct CpuCheck<P: CpuInfoProvider> {
    provider: P,
}

impl<P: CpuInfoProvider> CpuCheck<P> {
    pub fn new(provider: P) -> Self {
        Self { provider }
    }

    fn severity_for(usage_percent: f32) -> Severity {
        if usage_percent >= CRITICAL_PERCENT {
            Severity::Critical
        } else if usage_percent >= WARNING_PERCENT {
            Severity::Warning
        } else {
            Severity::Ok
        }
    }
}

impl<P: CpuInfoProvider> DiagnosticCheck for CpuCheck<P> {
    fn name(&self) -> &str {
        "cpu"
    }

    fn evaluate(&self) -> Result<CheckResult, DiagnosticError> {
        let usage_percent = self.provider.cpu_usage_percent();
        let severity = Self::severity_for(usage_percent);
        let message = format!("CPU usage is {usage_percent:.1}%");
        Ok(CheckResult::new(self.name(), severity, message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeCpuProvider(f32);
    impl CpuInfoProvider for FakeCpuProvider {
        fn cpu_usage_percent(&self) -> f32 {
            self.0
        }
    }

    #[test]
    fn reports_ok_under_threshold() {
        let check = CpuCheck::new(FakeCpuProvider(20.0));
        assert_eq!(check.evaluate().unwrap().severity, Severity::Ok);
    }

    #[test]
    fn reports_warning_at_threshold() {
        let check = CpuCheck::new(FakeCpuProvider(90.0));
        assert_eq!(check.evaluate().unwrap().severity, Severity::Warning);
    }

    #[test]
    fn reports_critical_when_saturated() {
        let check = CpuCheck::new(FakeCpuProvider(99.0));
        assert_eq!(check.evaluate().unwrap().severity, Severity::Critical);
    }
}
