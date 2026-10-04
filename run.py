import zipfile
import os

files_content = {
    "CLAUDE.md": """# Kubuntu Diagnostic Tool (kdiag) - Development Guide

## Project Overview
`kdiag` is a modular, high-performance CLI/TUI system diagnostic tool tailored for Kubuntu and KDE Plasma environments. It inspects kernel states, hardware metrics, systemd services, and desktop subsystem components (KWin, Wayland/X11, Plasma Shell, PipeWire, Baloo, APT/Snap locks).

## Architecture & Principles
- **Clean Architecture & Hexagonal Design:**
  - `core`: Diagnostic domain models, abstract port traits, and error hierarchies.
  - `collectors`: System data retrieval adapters (`sysfs`, `procfs`, `dbus`, `systemd`, `cli`).
  - `analyzers`: Rule evaluation engines (Rule Engine / Chain of Responsibility) converting raw metrics into prioritized findings.
  - `reporters`: Presentation adapters (CLI tables, JSON, Markdown) via the Visitor pattern.
- **SOLID & Idiomatic Rust:**
  - *Single Responsibility:* Each collector inspects a single, dedicated subsystem.
  - *Open/Closed:* New diagnostic checks and collectors register without modifying existing pipelines.
  - *Liskov Substitution:* Consistent failure and permission-handling contracts across all collectors.
  - *Interface Segregation:* Discrete contracts for collection, evaluation, and reporting.
  - *Dependency Inversion:* Analyzers depend strictly on trait abstractions rather than concrete OS interfaces.
- **Error Handling:** Domain-specific errors using `thiserror`; top-level CLI context managed with `anyhow`.
- **Zero-cost Abstractions:** Minimized heap allocations, zero-subprocess probing where feasible, non-blocking asynchronous I/O (`tokio`).

## Build & Test Commands
- Build: `cargo build --release`
- Run: `cargo run -- scan --all`
- Format Check: `cargo fmt --all -- --check`
- Lint: `cargo clippy --all-targets --all-features -- -D warnings`
- Test: `cargo test --all-features`
""",

    "AGENTS.md": """# Agent Personas & Task Matrix

## 1. System Collector Agent (`sys-collector`)
- **Focus:** Linux kernel interfaces (`/proc`, `/sys`), D-Bus protocols, and `systemd` daemon interaction.
- **Scope:** Hardware sensors (CPU, RAM, GPU, NVMe), KDE runtime services (KWin compositor, Plasma daemons).
- **Criteria:** Graceful degradation without root privileges, memory safety, zero-subprocess querying.

## 2. Analysis & Rule Engine Agent (`rule-engine`)
- **Focus:** Diagnostic evaluation rules, threshold checking, and anomaly identification.
- **Scope:** Kernel panics/OOM logs (`dmesg`), Wayland/Nvidia driver conflicts, zombie units, PipeWire buffer underruns.
- **Patterns:** Strategy, Rule Engine, Chain of Responsibility.

## 3. UI & Reporter Agent (`cli-reporter`)
- **Focus:** Terminal rendering (TUI/CLI via `ratatui` or `comfy-table`) and structured machine outputs (JSON, Markdown).
- **Patterns:** Visitor and Builder.

## 4. Code Quality & Performance Sentinel (`qa-sentinel`)
- **Focus:** Strict `clippy` adherence, memory profiling, lock contention prevention, and automated test coverage.
""",

    ".claude/agents/collector-expert.md": """---
name: collector-expert
description: Specialized in writing safe, non-blocking Linux and KDE subsystem collectors using Rust.
---
You are a systems developer specializing in Linux kernel interfaces, sysfs, procfs, and KDE D-Bus services.

### Directives:
1. Implement the `core::traits::Collector` trait across all subsystem modules.
2. Avoid spawning external CLI subshells (`lspci`, `cat`, `awk`); directly read from `/sys`, `/proc`, and type-safe D-Bus (`zbus`) endpoints.
3. Handle `PermissionDenied` errors gracefully as `DiagnosticFinding::warning` or `info` without crashing or panicking.
""",

    ".claude/agents/engine-architect.md": """---
name: engine-architect
description: Focuses on SOLID rules, Rule Engine design, and clean interfaces.
---
You are a systems architect enforcing SOLID design principles and clean domain modeling.

### Directives:
1. Define decoupled trait interfaces and domain aggregations inside `core/`.
2. Maintain a flexible, modular rule evaluation pipeline for diagnostic records.
3. Normalize all diagnostic evaluation outputs into the standard `DiagnosticReport` schema.
""",

    ".claude/skills/architecture-and-solid.md": """# Architecture & SOLID Guidelines for kdiag

## 1. Layered & Hexagonal Architecture
- **`core/` (Domain):** Zero external system dependencies. Houses pure domain types, error enums, and abstraction traits (`Collector`, `Analyzer`, `Reporter`).
- **`collectors/` (Inbound Adapters):** Zero business logic. Reads raw metrics from OS/D-Bus/sysfs and maps them into `DiagnosticMetric` domain objects.
- **`analyzers/` (Domain Logic):** Evaluates raw metrics through rule sets to yield structured findings (`Severity::Info`, `Severity::Warning`, `Severity::Critical`).
- **`reporters/` (Outbound Adapters):** Formats analysis results for CLI (tabular), JSON, or Markdown display.

## 2. SOLID Principles
- **Single Responsibility (SRP):** Each collector targets a single subsystem (`KWinCollector`, `PipewireCollector`, `AptLockCollector`).
- **Open/Closed (OCP):** Add new system checks by creating distinct `Collector` and `Rule` structs registered with the engine—never by modifying existing collectors.
- **Liskov Substitution (LSP):** All collectors fulfill the `async fn collect(&self)` contract uniformly; missing permissions return a structured status instead of panicking.
- **Interface Segregation (ISP):** Collection (`Collector`), Evaluation (`Analyzer`), and Formatting (`Reporter`) remain strictly isolated interfaces.
- **Dependency Inversion (DIP):** The diagnostic engine depends strictly on trait abstractions, not concrete OS/D-Bus implementations.

## 3. Design Patterns
- **Strategy Pattern:** Interchangeable collection algorithms and rule evaluations.
- **Visitor Pattern:** Traversing diagnostic reports across multiple output formatters.
- **Builder Pattern:** Assembling `DiagnosticReport` configurations and scan parameters.
- **Registry Pattern:** Dynamic runtime/compile-time collector registration.
""",

    ".claude/skills/kde-plasma-diagnostics.md": """# Kubuntu & KDE Plasma Diagnostic Checks

Standards for diagnosing Kubuntu desktop environments:

## 1. Display Server & KWin Compositor
- **Session Type:** Read `$XDG_SESSION_TYPE` (`wayland` / `x11`) and `$DESKTOP_SESSION`.
- **KWin Health:** Query `org.kde.KWin` via D-Bus.
  - Wayland crash tracking: Inspect `coredumpctl` or `journalctl /usr/bin/kwin_wayland -b -p 3`.
  - Compositor latency and freeze detection.

## 2. Audio & Media Pipeline (PipeWire / PulseAudio)
- **Service States:** Check user systemd units: `systemctl --user is-active pipewire pipewire-pulse wireplumber`.
- **Buffer Underruns:** Inspect recent system logs for ALSA/PipeWire xruns (`journalctl --user-unit=pipewire --since "1 hour ago"`).

## 3. KDE Background Daemons & Indexing
- **Baloo File Indexer:** Query `balooctl status` or D-Bus `org.kde.baloo` for stalled indexing, excessive CPU, or I/O thrashing.
- **KScreenLocker:** Check screen locker crash logs and failure states.

## 4. Package Management & Snap Integration
- **APT Lock Files:** Verify status of `/var/lib/dpkg/lock-frontend` and `/var/lib/apt/lists/lock`.
- **Broken Dependencies:** Check for unresolved packages via `/var/log/dpkg.log` and package manager states.
- **Snap Daemon:** Verify `snapd.service` status and identify hung snap refresh operations.
""",

    ".claude/skills/linux-system-probing.md": """# Linux Kernel & Hardware Probing Standards

Guidelines for efficient and secure system-level data collection:

## 1. Sysfs & Procfs Probing (Zero-Subprocess)
- Read files directly using `tokio::fs` or `std::fs` instead of invoking shell utilities (`cat`, `awk`, `grep`).
- **Memory:** Parse `/proc/meminfo` (`MemTotal`, `MemAvailable`, `SwapFree`) with minimal buffer allocations.
- **CPU Temperature & Frequency:** Read from `/sys/class/thermal/` and `/sys/devices/system/cpu/cpu*/cpufreq/scaling_cur_freq`.
- **Storage & SMART:** Inspect `/sys/block/*/queue/rotational` and hardware monitor sensors (`/sys/class/hwmon/`).

## 2. D-Bus Integration
- Utilize the `zbus` crate for asynchronous, type-safe IPC rather than spawning external CLI tools (`qdbus`).
- Query `systemd` states via `org.freedesktop.systemd1`.

## 3. Privilege Management & Graceful Degradation
- The tool must execute safely in non-root user mode.
- Unreadable privileged files (e.g., protected logs) must produce informative diagnostics rather than execution errors:
  ```rust
  DiagnosticFinding::info("Root privileges required to inspect full kernel log.")

```

""",
".claude/skills/rust-performance-and-safety.md": """# Rust Idioms, Performance & Clean Code

## 1. Error Handling (`thiserror` & `anyhow`)

* Use `thiserror` in domain modules (`core/`) for precise, typed error definitions:
```rust
#[derive(thiserror::Error, Debug)]
pub enum DiagnosticError {
    #[error("Subsystem unreachable: {0}")]
    SubsystemUnreachable(String),
    #[error("Parsing failed for {file}: {source}")]
    ParseError { file: String, source: std::io::Error },
    #[error("Permission denied accessing {0}")]
    AccessDenied(String),
}

```


* Use `anyhow::Result` only in the CLI boundary and top-level runner.
* Avoid `.unwrap()` and `.expect()` in production code paths; propagate with `?`.

## 2. Allocation & Concurrency

* Execute independent collectors concurrently using `tokio::task::JoinSet` or `futures::future::join_all`.
* Preallocate buffers where capacities are known (`String::with_capacity`, `Vec::with_capacity`).
* Prefer borrowed slices (`&str`, `&Path`) over owned allocations in read pathways.
* Use `BufReader` for sequential file parsing.

## 3. Clean Code & Readability

* Keep functions focused and concise (under 30 lines where practical).
* Centralize file paths and magic numbers into constants or configuration structs.
* Document all public traits, structs, and functions using Rustdoc (`///`).
"""
}
