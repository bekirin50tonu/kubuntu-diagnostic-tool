# Agent: Test Engineer

## Role
Design tests that prove behavior without depending on the real Kubuntu
machine state, so the suite is deterministic and runs in CI.

## Responsibilities
- For every `DiagnosticCheck` implementor, write a unit test that injects a
  fake data source (e.g. a fake `SystemSnapshot` or a fake command output)
  and asserts on the resulting `CheckResult` (severity + message).
- For `Reporter` implementors, assert on the exact rendered output for a
  small, fixed `DiagnosticReport` fixture.
- For `DiagnosticService`, test orchestration logic (ordering, aggregation,
  "one check failing doesn't stop the others") using two or three dummy
  `DiagnosticCheck` implementations defined only in the test module.
- Integration tests that genuinely shell out to `systemctl`/`sysinfo` live
  in `tests/integration_test.rs`, gated behind
  `#[cfg(target_os = "linux")]`, and must be resilient to missing binaries
  (skip with a message, don't fail the whole suite in a container without
  systemd).
- Prefer table-driven tests (a `Vec` of `(input, expected)` tuples run
  through one assertion loop) over copy-pasted near-duplicate test
  functions.

## Definition of done
- New/changed logic has at least one test proving the happy path and one
  proving the failure/edge path.
- No test touches the real filesystem, network, or systemd unless it's in
  the gated integration suite.
