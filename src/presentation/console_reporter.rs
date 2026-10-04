use crate::domain::{DiagnosticError, DiagnosticReport, Reporter};
use std::fmt::Write as _;

/// Renders a report as human-readable, colon-aligned plain text.
pub struct ConsoleReporter;

impl Reporter for ConsoleReporter {
    fn render(&self, report: &DiagnosticReport) -> Result<String, DiagnosticError> {
        let mut output = String::new();

        writeln!(
            output,
            "Kubuntu Diagnostic Report — {}",
            report.generated_at
        )
        .map_err(render_failure)?;
        writeln!(output, "Overall status: {}", report.overall_severity())
            .map_err(render_failure)?;
        writeln!(output).map_err(render_failure)?;

        for result in &report.results {
            writeln!(
                output,
                "[{:<8}] {:<10} {}",
                result.severity, result.check_name, result.message
            )
            .map_err(render_failure)?;
            if let Some(details) = &result.details {
                writeln!(output, "             └─ {details}").map_err(render_failure)?;
            }
        }

        Ok(output)
    }
}

fn render_failure(error: std::fmt::Error) -> DiagnosticError {
    DiagnosticError::RenderFailure {
        format: "console".to_string(),
        reason: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{CheckResult, Severity};

    #[test]
    fn renders_every_result_and_the_overall_status() {
        let report = DiagnosticReport::new(
            "2026-01-01T00:00:00Z",
            vec![
                CheckResult::new("disk", Severity::Ok, "nominal"),
                CheckResult::new("memory", Severity::Warning, "high usage")
                    .with_details("swap usage is 12.0%"),
            ],
        );

        let rendered = ConsoleReporter.render(&report).unwrap();

        assert!(rendered.contains("Overall status: WARNING"));
        assert!(rendered.contains("disk"));
        assert!(rendered.contains("memory"));
        assert!(rendered.contains("swap usage is 12.0%"));
    }
}
