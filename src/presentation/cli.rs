use clap::{Parser, ValueEnum};

/// A Kubuntu system diagnostic tool.
#[derive(Debug, Parser)]
#[command(name = "kubuntu-diagnostic-tool", version, about)]
pub struct Cli {
    /// Output format for the report.
    #[arg(long, value_enum, default_value_t = OutputFormat::Console)]
    pub format: OutputFormat,

    /// Exit with a non-zero status if any check is at or above this
    /// severity. Defaults to "error" (warnings alone do not fail the run).
    #[arg(long, value_enum, default_value_t = FailOn::Error)]
    pub fail_on: FailOn,

    /// Show a KDE system tray icon with CPU/RAM/Disk usage.
    #[arg(long)]
    pub tray: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Console,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum FailOn {
    Warning,
    Error,
    Critical,
}

impl FailOn {
    pub fn as_severity(self) -> crate::domain::Severity {
        use crate::domain::Severity;
        match self {
            Self::Warning => Severity::Warning,
            Self::Error => Severity::Error,
            Self::Critical => Severity::Critical,
        }
    }
}
