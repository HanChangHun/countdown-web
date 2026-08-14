use tauri_plugin_window_state::StateFlags;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Persist the window size (but NOT its position) so a remembered window can
    // never be restored off-screen after a monitor is disconnected/rearranged —
    // it always reopens centered (tauri.conf `center: true`) at the saved size.
    // DECORATIONS is also excluded: the frameless custom titlebar must not be
    // overridden by a previously saved `decorated: true` state.
    let window_state = tauri_plugin_window_state::Builder::default()
        .with_state_flags(StateFlags::all() & !(StateFlags::POSITION | StateFlags::DECORATIONS))
        .build();

    tauri::Builder::default()
        .plugin(window_state)
        // Open external links (the credit link) in the system browser.
        .plugin(tauri_plugin_opener::init())
        // Secure auto-update (signed GitHub releases) + relaunch after install.
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        // Desktop notification when a countdown hits zero.
        .plugin(tauri_plugin_notification::init())
        .setup(|_app| {
            std::thread::spawn(cleanup_stale_updater_temp_dirs);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running countdown application");
}

// Tauri's NSIS updater extracts each update into %TEMP% and never removes it
// afterward (tauri-apps/tauri#11862, unfixed upstream), so every update leaves
// a few MB behind. By the time this runs, any update that led to the current
// launch has already finished, so every matching folder here is stale.
fn cleanup_stale_updater_temp_dirs() {
    let Ok(entries) = std::fs::read_dir(std::env::temp_dir()) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with("Countdown-") && name.contains("-updater-") && entry.path().is_dir() {
            let _ = std::fs::remove_dir_all(entry.path());
        }
    }
}
