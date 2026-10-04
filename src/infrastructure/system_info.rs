use std::cell::RefCell;
use std::rc::Rc;
use sysinfo::{Disks, System};

/// Disk usage for a single mounted filesystem, already reduced to the one
/// number `DiskCheck` cares about.
#[derive(Debug, Clone, PartialEq)]
pub struct DiskUsage {
    pub mount_point: String,
    pub used_percent: f32,
}

/// A small, focused port `DiskCheck` depends on instead of `sysinfo`
/// directly. Interface Segregation: `DiskCheck` only ever needs this one
/// method, so that's all the trait exposes.
pub trait DiskInfoProvider {
    fn disk_usages(&self) -> Vec<DiskUsage>;
}

/// RAM + swap usage, already reduced to percentages.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MemoryUsage {
    pub used_percent: f32,
    pub swap_used_percent: f32,
}

pub trait MemoryInfoProvider {
    fn memory_usage(&self) -> MemoryUsage;
}

pub trait CpuInfoProvider {
    /// Global CPU usage as a percentage, averaged across all cores.
    fn cpu_usage_percent(&self) -> f32;
}

/// The real-world implementation of all three providers, backed by a single
/// shared `sysinfo::System` so we don't pay the cost of scanning `/proc`
/// three times per diagnostic run.
///
/// `RefCell` gives interior mutability: `sysinfo` needs `&mut self` to
/// refresh, but `DiagnosticCheck::evaluate` only gets `&self` (checks must
/// not need external mutable access to be run). This is an infrastructure
/// detail entirely local to this adapter — it never leaks into the domain.
pub struct SysinfoSnapshot {
    system: RefCell<System>,
    disks: RefCell<Disks>,
}

impl SysinfoSnapshot {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        let disks = Disks::new_with_refreshed_list();
        Self {
            system: RefCell::new(system),
            disks: RefCell::new(disks),
        }
    }

    /// Refresh the system and disks information.
    pub fn refresh(&self) {
        self.system.borrow_mut().refresh_all();
        self.disks.borrow_mut().refresh_list();
    }
}

impl Default for SysinfoSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

impl DiskInfoProvider for SysinfoSnapshot {
    fn disk_usages(&self) -> Vec<DiskUsage> {
        self.disks
            .borrow()
            .iter()
            .filter_map(|disk| {
                let total = disk.total_space();
                if total == 0 {
                    return None;
                }
                let used = total.saturating_sub(disk.available_space());
                let used_percent = (used as f64 / total as f64 * 100.0) as f32;
                Some(DiskUsage {
                    mount_point: disk.mount_point().to_string_lossy().into_owned(),
                    used_percent,
                })
            })
            .collect()
    }
}

impl MemoryInfoProvider for SysinfoSnapshot {
    fn memory_usage(&self) -> MemoryUsage {
        let system = self.system.borrow();
        let used_percent = percent(system.used_memory(), system.total_memory());
        let swap_used_percent = percent(system.used_swap(), system.total_swap());
        MemoryUsage {
            used_percent,
            swap_used_percent,
        }
    }
}

impl CpuInfoProvider for SysinfoSnapshot {
    fn cpu_usage_percent(&self) -> f32 {
        self.system.borrow().global_cpu_info().cpu_usage()
    }
}

// Blanket impls so `Rc<SysinfoSnapshot>` (used by the check factory to
// share one machine scan across three checks) satisfies the same provider
// traits as an owned `SysinfoSnapshot`, without any check needing to know
// it's actually looking at a shared, reference-counted instance.
impl<T: DiskInfoProvider> DiskInfoProvider for Rc<T> {
    fn disk_usages(&self) -> Vec<DiskUsage> {
        (**self).disk_usages()
    }
}

impl<T: MemoryInfoProvider> MemoryInfoProvider for Rc<T> {
    fn memory_usage(&self) -> MemoryUsage {
        (**self).memory_usage()
    }
}

impl<T: CpuInfoProvider> CpuInfoProvider for Rc<T> {
    fn cpu_usage_percent(&self) -> f32 {
        (**self).cpu_usage_percent()
    }
}

fn percent(part: u64, total: u64) -> f32 {
    if total == 0 {
        0.0
    } else {
        (part as f64 / total as f64 * 100.0) as f32
    }
}
