use crate::domain::{CheckResult, DiagnosticCheck, DiagnosticError, Severity};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

/// A small port around "can we reach the network", so `NetworkCheck` never
/// has to know about sockets, timeouts, or which address it's probing.
pub trait NetworkProbe {
    fn is_reachable(&self) -> bool;
}

/// The real-world probe: attempts a short TCP connection to a well-known,
/// highly available public resolver. This deliberately avoids DNS lookups
/// (which would add a second failure mode to disentangle) by connecting to
/// a raw IP address.
pub struct TcpNetworkProbe {
    target: SocketAddr,
    timeout: Duration,
}

impl TcpNetworkProbe {
    pub fn new(target: SocketAddr, timeout: Duration) -> Self {
        Self { target, timeout }
    }
}

impl Default for TcpNetworkProbe {
    fn default() -> Self {
        // 1.1.1.1:53 — Cloudflare's public DNS resolver, chosen for high
        // availability rather than any DNS-specific behavior.
        Self::new(
            "1.1.1.1:53".parse().expect("hardcoded address is valid"),
            Duration::from_secs(2),
        )
    }
}

impl NetworkProbe for TcpNetworkProbe {
    fn is_reachable(&self) -> bool {
        TcpStream::connect_timeout(&self.target, self.timeout).is_ok()
    }
}

/// Reports whether the machine currently has outbound network connectivity.
pub struct NetworkCheck<P: NetworkProbe> {
    probe: P,
}

impl<P: NetworkProbe> NetworkCheck<P> {
    pub fn new(probe: P) -> Self {
        Self { probe }
    }
}

impl<P: NetworkProbe> DiagnosticCheck for NetworkCheck<P> {
    fn name(&self) -> &str {
        "network"
    }

    fn evaluate(&self) -> Result<CheckResult, DiagnosticError> {
        let result = if self.probe.is_reachable() {
            CheckResult::new(self.name(), Severity::Ok, "outbound connectivity is up")
        } else {
            CheckResult::new(
                self.name(),
                Severity::Error,
                "outbound connectivity probe failed",
            )
        };
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeProbe(bool);
    impl NetworkProbe for FakeProbe {
        fn is_reachable(&self) -> bool {
            self.0
        }
    }

    #[test]
    fn reports_ok_when_reachable() {
        let check = NetworkCheck::new(FakeProbe(true));
        assert_eq!(check.evaluate().unwrap().severity, Severity::Ok);
    }

    #[test]
    fn reports_error_when_unreachable() {
        let check = NetworkCheck::new(FakeProbe(false));
        assert_eq!(check.evaluate().unwrap().severity, Severity::Error);
    }
}
