//! The infrastructure layer: adapters between real system data sources
//! (`sysinfo`, `systemctl`, TCP probes) and the domain's `DiagnosticCheck`
//! / `Clock` traits.
//!
//! Everything here may depend on `domain`. Nothing in `domain` or
//! `application` ever depends back on this module — that dependency
//! direction is what makes the domain testable without a real machine.

pub mod checks;
pub mod system_clock;
pub mod system_info;

pub use system_clock::SystemClock;
