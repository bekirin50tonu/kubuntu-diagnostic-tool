//! Binary entry point: CLI parsing, calling into the library's composition
//! logic, and choosing how to render + exit. This is the only file allowed
//! to know about every layer at once for wiring purposes.

use clap::Parser;
use kubuntu_diagnostic_tool::presentation::{Cli, ConsoleReporter, JsonReporter, OutputFormat};
use kubuntu_diagnostic_tool::{domain, run_default_diagnostics};
use std::process::ExitCode;
use kubuntu_diagnostic_tool::presentation::tray::run_tray;

fn main() -> ExitCode {
    let cli = Cli::parse();

    if cli.tray {
        if let Err(e) = run_tray() {
            eprintln!("Failed to run tray: {e}");
            return ExitCode::FAILURE;
        }
        return ExitCode::SUCCESS;
    }

    let report = run_default_diagnostics();

    let reporter: Box<dyn domain::Reporter> = match cli.format {
        OutputFormat::Console => Box::new(ConsoleReporter),
        OutputFormat::Json => Box::new(JsonReporter),
    };

    match reporter.render(&report) {
        Ok(rendered) => println!("{rendered}"),
        Err(error) => {
            eprintln!("failed to render diagnostic report: {error}");
            return ExitCode::FAILURE;
        }
    }

    if report.overall_severity() >= cli.fail_on.as_severity() {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
