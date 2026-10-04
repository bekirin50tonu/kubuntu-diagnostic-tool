use crate::domain::DiagnosticCheck;
use crate::infrastructure::checks::{CpuCheck, DiskCheck, MemoryCheck, NetworkCheck, ServiceCheck};
use crate::infrastructure::checks::cleanup_check::{CleanupCheck, RealCleanupProvider};
use crate::infrastructure::checks::network_check::TcpNetworkProbe;
use crate::infrastructure::checks::service_check::SystemctlFailedUnitsProvider;
use crate::infrastructure::checks::apt_cache_check::{AptCacheCheck, RealAptCacheProvider};
use crate::infrastructure::system_info::SysinfoSnapshot;
use std::rc::Rc;

/// The Factory: builds the standard set of checks this tool ships with.
///
/// This is the *only* place that needs to change to add or remove a check
/// from the default run — `DiagnosticService` never changes (Open/Closed
/// Principle). Disk, memory and CPU checks share one `SysinfoSnapshot` via
/// `Rc` so the machine is only scanned once per run.
pub fn build_default_checks() -> Vec<Box<dyn DiagnosticCheck>> {
    let snapshot = Rc::new(SysinfoSnapshot::new());

    vec![
        Box::new(DiskCheck::new(Rc::clone(&snapshot))),
        Box::new(MemoryCheck::new(Rc::clone(&snapshot))),
        Box::new(CpuCheck::new(Rc::clone(&snapshot))),
        Box::new(NetworkCheck::new(TcpNetworkProbe::default())),
        Box::new(ServiceCheck::new(SystemctlFailedUnitsProvider)),
        Box::new(CleanupCheck::new(RealCleanupProvider)),
        Box::new(AptCacheCheck::new(RealAptCacheProvider)),
    ]
}
