use crate::domain::{CheckResult, Clock, DiagnosticCheck, DiagnosticReport, Severity};

/// Orchestrates a set of `DiagnosticCheck`s into a single `DiagnosticReport`.
///
/// This is the only place that knows "how a diagnostic run works" (run
/// every check, don't let one failure abort the rest, stamp the report
/// with a timestamp). It knows nothing about *which* checks exist or *how*
/// a report gets displayed — those are injected, which is what keeps this
/// type open for extension without modification.
pub struct DiagnosticService {
    checks: Vec<Box<dyn DiagnosticCheck>>,
    clock: Box<dyn Clock>,
}

impl DiagnosticService {
    /// Both dependencies are injected (Dependency Inversion) so the
    /// composition root controls exactly which checks run and where "now"
    /// comes from — tests can substitute fakes for both.
    pub fn new(checks: Vec<Box<dyn DiagnosticCheck>>, clock: Box<dyn Clock>) -> Self {
        Self { checks, clock }
    }

    /// Run every registered check and collect the results into a report.
    ///
    /// A check that returns `Err` is not fatal to the run: it is turned
    /// into a `CheckResult` with `Severity::Error` so the operator still
    /// sees *something* went wrong with that specific check, while every
    /// other check still gets a chance to run.
    pub fn run_all(&self) -> DiagnosticReport {
        let results = self
            .checks
            .iter()
            .map(|check| self.evaluate_or_report_failure(check.as_ref()))
            .collect();

        DiagnosticReport::new(self.clock.now_rfc3339(), results)
    }

    fn evaluate_or_report_failure(&self, check: &dyn DiagnosticCheck) -> CheckResult {
        match check.evaluate() {
            Ok(result) => result,
            Err(error) => CheckResult::new(check.name(), Severity::Error, error.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DiagnosticError;

    struct FixedClock(&'static str);
    impl Clock for FixedClock {
        fn now_rfc3339(&self) -> String {
            self.0.to_string()
        }
    }

    struct AlwaysOkCheck;
    impl DiagnosticCheck for AlwaysOkCheck {
        fn name(&self) -> &str {
            "always_ok"
        }
        fn evaluate(&self) -> Result<CheckResult, DiagnosticError> {
            Ok(CheckResult::new(self.name(), Severity::Ok, "nominal"))
        }
    }

    struct AlwaysFailingCheck;
    impl DiagnosticCheck for AlwaysFailingCheck {
        fn name(&self) -> &str {
            "always_failing"
        }
        fn evaluate(&self) -> Result<CheckResult, DiagnosticError> {
            Err(DiagnosticError::DataSourceUnavailable {
                check_name: self.name().to_string(),
                reason: "simulated failure".to_string(),
            })
        }
    }

    #[test]
    fn run_all_stamps_the_report_with_the_injected_clock() {
        let service = DiagnosticService::new(
            vec![Box::new(AlwaysOkCheck)],
            Box::new(FixedClock("2026-01-01T00:00:00Z")),
        );
        let report = service.run_all();
        assert_eq!(report.generated_at, "2026-01-01T00:00:00Z");
    }

    #[test]
    fn one_failing_check_does_not_stop_the_others() {
        let service = DiagnosticService::new(
            vec![Box::new(AlwaysFailingCheck), Box::new(AlwaysOkCheck)],
            Box::new(FixedClock("2026-01-01T00:00:00Z")),
        );
        let report = service.run_all();

        assert_eq!(report.results.len(), 2, "both checks must produce a result");
        assert_eq!(report.results[0].severity, Severity::Error);
        assert_eq!(report.results[1].severity, Severity::Ok);
    }

    #[test]
    fn empty_check_list_produces_an_ok_report() {
        let service = DiagnosticService::new(vec![], Box::new(FixedClock("2026-01-01T00:00:00Z")));
        let report = service.run_all();
        assert!(report.results.is_empty());
        assert_eq!(report.overall_severity(), Severity::Ok);
    }
}
