use crate::domain::Clock;
use chrono::Utc;

/// The real-world `Clock` implementation, backed by the system clock.
///
/// This is an Adapter: it exists purely to let `main.rs` hand
/// `DiagnosticService` something that satisfies `domain::Clock` without the
/// domain ever importing `chrono` itself.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_rfc3339(&self) -> String {
        Utc::now().to_rfc3339()
    }
}
