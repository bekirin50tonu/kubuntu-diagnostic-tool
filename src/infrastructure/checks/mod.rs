pub mod cpu_check;
pub mod disk_check;
pub mod memory_check;
pub mod network_check;
pub mod registry;
pub mod service_check;
pub mod cleanup_check;
pub mod apt_cache_check;

pub use cpu_check::CpuCheck;
pub use disk_check::DiskCheck;
pub use memory_check::MemoryCheck;
pub use network_check::NetworkCheck;
pub use registry::build_default_checks;
pub use service_check::ServiceCheck;
pub use cleanup_check::CleanupCheck;
pub use apt_cache_check::AptCacheCheck;