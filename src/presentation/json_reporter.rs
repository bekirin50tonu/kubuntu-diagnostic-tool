use crate::domain::{DiagnosticError, DiagnosticReport, Reporter};

/// Renders a report as pretty-printed JSON, for machine consumption
/// (monitoring pipelines, CI, other tools).
pub struct JsonReporter;

impl Reporter for JsonReporter {
    fn render(&self, report: &DiagnosticReport) -> Result<String, DiagnosticError> {
        serde_json::to_string_pretty(report).map_err(|error| DiagnosticError::RenderFailure {
            format: "json".to_string(),
            reason: error.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{CheckResult, Severity};

    #[test]
    fn renders_valid_json_containing_every_result() {
        let report = DiagnosticReport::new(
            "2026-01-01T00:00:00Z",
            vec![CheckResult::new("disk", Severity::Critical, "almost full")],
        );

        let rendered = JsonReporter.render(&report).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&rendered).unwrap();

        assert_eq!(parsed["results"][0]["check_name"], "disk");
        assert_eq!(parsed["results"][0]["severity"], "critical");
    }
}
