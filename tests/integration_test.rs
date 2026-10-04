//! Integration tests that touch real system state. Gated to Linux and
//! written to be resilient to environments without systemd (e.g. minimal
//! containers) — a missing `systemctl` skips the assertion instead of
//! failing the whole suite.

#[cfg(target_os = "linux")]
#[test]
fn disk_and_memory_checks_run_against_the_real_machine() {
    use kubuntu_diagnostic_tool::run_default_diagnostics;

    let report = run_default_diagnostics();

    let names: Vec<&str> = report
        .results
        .iter()
        .map(|r| r.check_name.as_str())
        .collect();
    assert!(names.contains(&"disk"));
    assert!(names.contains(&"memory"));
    assert!(names.contains(&"cpu"));
}
