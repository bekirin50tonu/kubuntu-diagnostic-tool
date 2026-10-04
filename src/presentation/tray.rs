use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use std::iter;
use ctrlc;
use ksni::{Tray, blocking::TrayMethods};
use crate::infrastructure::system_info::{DiskInfoProvider, MemoryInfoProvider, CpuInfoProvider, SysinfoSnapshot};

/// Progress bar using Unicode block elements (similar to UI_DESIGN.md)
fn progress_bar(percent: f32) -> String {
    const FULL: char = '\u{2588}'; // █
    const LOW: char = '\u{2591}';  // ░
    const TOTAL_WIDTH: usize = 6;
    let filled = ((percent / 100.0) * TOTAL_WIDTH as f32).round() as usize;
    let empty = TOTAL_WIDTH.saturating_sub(filled);
    let filled_str: String = iter::repeat_n(FULL, filled).collect();
    let empty_str: String = iter::repeat_n(LOW, empty).collect();
    format!("{}{}", filled_str, empty_str)
}

/// Shared state for the tray percentages.
struct SharedState {
    cpu_percent: u32,
    ram_percent: u32,
    disk_percent: u32,
}

/// The tray item that holds a reference to the shared state.
struct DiagnosticTray {
    shared_state: Arc<Mutex<SharedState>>,
}

impl Tray for DiagnosticTray {
    fn id(&self) -> String {
        "kubuntu-diagnostic-tool".into()
    }

    fn title(&self) -> String {
        "Kubuntu Diagnostic Tool".into()
    }

    fn icon_name(&self) -> String {
        // Try to use the Breeze preferences-system symbolic icon.
        "preferences-system".into()
    }

    fn menu(&self) -> Vec<ksni::menu::MenuItem<Self>> {
        // Read the current percentages from shared state.
        let state = self.shared_state.lock().unwrap();
        let cpu = state.cpu_percent;
        let ram = state.ram_percent;
        let disk = state.disk_percent;

        let cpu_label = format!("CPU   {} {:>3}%", progress_bar(cpu as f32), cpu);
        let ram_label = format!("RAM   {} {:>3}%", progress_bar(ram as f32), ram);
        let disk_label = format!("Disk  {} {:>3}%", progress_bar(disk as f32), disk);

        let mut menu = Vec::new();
        menu.push(ksni::menu::MenuItem::Standard(ksni::menu::StandardItem {
            label: cpu_label,
            enabled: true,
            visible: true,
            icon_name: String::new(),
            icon_data: Vec::new(),
            shortcut: Vec::new(),
            disposition: ksni::menu::Disposition::Normal,
            activate: Box::new(|_| {}),
        }));
        menu.push(ksni::menu::MenuItem::Standard(ksni::menu::StandardItem {
            label: ram_label,
            enabled: true,
            visible: true,
            icon_name: String::new(),
            icon_data: Vec::new(),
            shortcut: Vec::new(),
            disposition: ksni::menu::Disposition::Normal,
            activate: Box::new(|_| {}),
        }));
        menu.push(ksni::menu::MenuItem::Standard(ksni::menu::StandardItem {
            label: disk_label,
            enabled: true,
            visible: true,
            icon_name: String::new(),
            icon_data: Vec::new(),
            shortcut: Vec::new(),
            disposition: ksni::menu::Disposition::Normal,
            activate: Box::new(|_| {}),
        }));
        menu.push(ksni::menu::MenuItem::Separator);
        menu.push(ksni::menu::MenuItem::Standard(ksni::menu::StandardItem {
            label: "Open".into(),
            enabled: true,
            visible: true,
            icon_name: String::new(),
            icon_data: Vec::new(),
            shortcut: Vec::new(),
            disposition: ksni::menu::Disposition::Normal,
            activate: Box::new(|_| {
                println!("Open action triggered (main window not implemented in tray-only mode)");
            }),
        }));
        menu.push(ksni::menu::MenuItem::Standard(ksni::menu::StandardItem {
            label: "Quit".into(),
            enabled: true,
            visible: true,
            icon_name: String::new(),
            icon_data: Vec::new(),
            shortcut: Vec::new(),
            disposition: ksni::menu::Disposition::Normal,
            activate: Box::new(|_| {
                std::process::exit(0);
            }),
        }));
        menu
    }
}

pub fn run_tray() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Shared state for the percentages, initialized to zero.
    let shared_state = Arc::new(Mutex::new(SharedState {
        cpu_percent: 0,
        ram_percent: 0,
        disk_percent: 0,
    }));
    let shared_state_for_update = Arc::clone(&shared_state);

    // Create the tray instance.
    let tray = DiagnosticTray {
        shared_state: Arc::clone(&shared_state),
    };

    // Spawn the tray service (blocking) and get a handle.
    let handle = tray.spawn()?;

    // Clone the handle and shared state for the background update thread.
    let handle_clone = handle.clone();
    let stop_update = Arc::new(AtomicBool::new(false));
    let stop_update_clone = Arc::clone(&stop_update);

    // Spawn a background thread to update the percentages every 500ms (2Hz).
    let update_thread = thread::spawn(move || {
        while !stop_update_clone.load(Ordering::Relaxed) {
            // Create a fresh sysinfo snapshot to get current usage.
            let snapshot = SysinfoSnapshot::new();
            let cpu_percent = snapshot.cpu_usage_percent();
            let mem_usage = snapshot.memory_usage();
            let ram_percent = mem_usage.used_percent;
            let disk_usages = snapshot.disk_usages();
            let disk_percent = disk_usages
                .iter()
                .max_by(|a, b| a.used_percent.total_cmp(&b.used_percent))
                .map(|du| du.used_percent)
                .unwrap_or(0.0);

            // Update the shared state.
            let mut state = shared_state_for_update.lock().unwrap();
            state.cpu_percent = cpu_percent as u32;
            state.ram_percent = ram_percent as u32;
            state.disk_percent = disk_percent as u32;

            // Notify the tray that the menu should be redrawn.
            // We pass an empty closure because the tray's menu reads from shared_state.
            let _ = handle_clone.update(|_tray| {});

            // Sleep for 500ms.
            thread::sleep(Duration::from_millis(500));
        }
    });

    // Set up Ctrl+C handler to shut down the tray and stop the update thread.
    let running = Arc::new(AtomicBool::new(true));
    let r = Arc::clone(&running);
    let handle_for_ctrlc = handle.clone();
    let stop_update_for_ctrlc = Arc::clone(&stop_update);
    ctrlc::set_handler(move || {
        println!("\nReceived Ctrl+C, shutting down...");
        r.store(false, Ordering::Relaxed);
        // Shut down the tray service (blocking shutdown).
        handle_for_ctrlc.shutdown().wait();
        // Signal the update thread to stop.
        stop_update_for_ctrlc.store(true, Ordering::Relaxed);
    })?;

    // Wait until the running flag is set to false (by Ctrl+C handler).
    while running.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(100));
    }

    // Wait for the update thread to finish.
    let _ = update_thread.join();

    Ok(())
}