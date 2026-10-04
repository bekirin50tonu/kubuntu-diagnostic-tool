//! Tauri GUI entry point for KDiag

use kubuntu_diagnostic_tool::domain::DiagnosticReport;
use kubuntu_diagnostic_tool::run_default_diagnostics;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

#[derive(Default)]
struct AppState {
    last_report: Mutex<Option<DiagnosticReportView>>,
}

#[derive(Serialize, Deserialize, Clone)]
struct CheckResultView {
    check_name: String,
    severity: String,
    message: String,
    details: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct DiagnosticReportView {
    generated_at: String,
    overall_severity: String,
    results: Vec<CheckResultView>,
}

impl From<DiagnosticReport> for DiagnosticReportView {
    fn from(report: DiagnosticReport) -> Self {
        Self {
            generated_at: report.generated_at,
            overall_severity: format!("{:?}", report.overall_severity()),
            results: report
                .results
                .into_iter()
                .map(|r| CheckResultView {
                    check_name: r.check_name,
                    severity: format!("{:?}", r.severity),
                    message: r.message,
                    details: r.details,
                })
                .collect(),
        }
    }
}

/// Global system metrics snapshot for the dashboard.
#[derive(Serialize, Deserialize, Clone)]
struct SystemInfoView {
    hostname: String,
    kernel_version: String,
    os_version: String,
    uptime: String,
    cpu_brand: String,
    cpu_count: usize,
    cpu_usage: f32,
    total_memory: u64,
    used_memory: u64,
    total_swap: u64,
    used_swap: u64,
    total_disk: u64,
    used_disk: u64,
}

fn format_duration(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;
    let secs = seconds % 60;

    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else if hours > 0 {
        format!("{hours}h {minutes}m {secs}s")
    } else {
        format!("{minutes}m {secs}s")
    }
}

#[tauri::command]
async fn run_diagnostics(state: State<'_, AppState>) -> Result<DiagnosticReportView, String> {
    let report = run_default_diagnostics();
    let view: DiagnosticReportView = report.into();
    *state.last_report.lock().unwrap() = Some(view.clone());
    Ok(view)
}

#[tauri::command]
async fn get_last_report(state: State<'_, AppState>) -> Result<Option<DiagnosticReportView>, String> {
    Ok(state.last_report.lock().unwrap().clone())
}

/// Runs a systemctl action (status/start/stop/restart) on a unit and returns
/// its output. Privileged actions will fail without elevation; that error is
/// surfaced to the UI rather than swallowed.
#[tauri::command]
async fn run_systemctl_command(action: String, unit: String) -> Result<String, String> {
    use std::process::Command;

    let output = Command::new("systemctl")
        .args([action.as_str(), unit.as_str()])
        .output()
        .map_err(|e| format!("failed to run systemctl: {e}"))?;

    let text = if output.status.success() {
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        String::from_utf8_lossy(&output.stderr).to_string()
    };

    Ok(text)
}

#[tauri::command]
async fn get_system_info() -> Result<SystemInfoView, String> {
    use sysinfo::{Disks, System};

    let mut system = System::new_all();
    system.refresh_all();
    let disks = Disks::new_with_refreshed_list();

    let use_bytes = move |p: u64, t: u64| {
        if t == 0 {
            0.0f32
        } else {
            (p as f64 / t as f64 * 100.0) as f32
        }
    };

    let total_memory = system.total_memory();
    let used_memory = system.used_memory();
    let total_swap = system.total_swap();
    let used_swap = system.used_swap();

    let total_disk: u64 = disks.iter().map(|d| d.total_space()).sum();
    let available_disk: u64 = disks.iter().map(|d| d.available_space()).sum();
    let used_disk = total_disk.saturating_sub(available_disk);

    let cpus = system.cpus();
    let cpu_brand = cpus.first().map(|c| c.brand().to_string()).unwrap_or_default();
    // sysinfo 0.30 reports per-CPU usage; average it across cores.
    let cpu_usage: f32 = if cpus.is_empty() {
        0.0
    } else {
        cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len() as f32
    };

    let _ = use_bytes; // silence unused warning; kept for per-metric reuse

    Ok(SystemInfoView {
        hostname: System::host_name().unwrap_or_default(),
        kernel_version: System::kernel_version().unwrap_or_default(),
        os_version: System::long_os_version().unwrap_or_default(),
        uptime: format_duration(System::uptime()),
        cpu_brand,
        cpu_count: cpus.len(),
        cpu_usage,
        total_memory,
        used_memory,
        total_swap,
        used_swap,
        total_disk,
        used_disk,
    })
}

#[tauri::command]
async fn get_failed_services() -> Result<Vec<String>, String> {
    use std::process::Command;

    let output = Command::new("systemctl")
        .args(["--failed", "--plain", "--no-legend"])
        .output()
        .map_err(|e| format!("failed to run systemctl: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .map(str::to_string)
        .collect())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            run_diagnostics,
            get_last_report,
            run_systemctl_command,
            get_system_info,
            get_failed_services,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}