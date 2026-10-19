//! WhatsInside – ľahký prehľad o tom, čo je v tvojom PC. Od WheresTheApp.
//!
//! Everything here is read-only. When the window is closed nothing keeps
//! running: no service, no tray icon, no background thread.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![cfg_attr(not(windows), allow(dead_code))]

mod autostart;
mod catalog;
mod hw;
mod known;
mod procs;
mod storage;
mod system;

use std::sync::Mutex;
use tauri::State;

#[derive(Default)]
struct AppState {
    sampler: Mutex<procs::Sampler>,
    scanner: storage::Scanner,
}

#[tauri::command]
async fn procs_snapshot(state: State<'_, AppState>) -> Result<procs::Snapshot, String> {
    Ok(state.sampler.lock().map_err(|e| e.to_string())?.snapshot())
}

#[tauri::command]
async fn autostart_list() -> Vec<autostart::Item> {
    autostart::list()
}

#[tauri::command]
async fn hw_scan() -> hw::HwReport {
    hw::scan()
}

#[tauri::command]
async fn storage_drives() -> Vec<storage::Drive> {
    storage::drives()
}

#[tauri::command]
async fn storage_start(state: State<'_, AppState>, root: String) -> Result<(), String> {
    if !std::path::Path::new(&root).is_dir() {
        return Err("Tento priečinok neexistuje.".into());
    }
    state.scanner.start(root);
    Ok(())
}

#[tauri::command]
async fn storage_cancel(state: State<'_, AppState>) -> Result<(), String> {
    state.scanner.cancel();
    Ok(())
}

#[tauri::command]
async fn storage_status(state: State<'_, AppState>) -> Result<storage::Status, String> {
    Ok(state.scanner.status())
}

#[tauri::command]
async fn storage_node(
    state: State<'_, AppState>,
    id: u32,
) -> Result<Option<storage::Listing>, String> {
    Ok(state.scanner.with_tree(|t| t.describe(id)).flatten())
}

#[tauri::command]
async fn storage_top_files(state: State<'_, AppState>) -> Result<Vec<storage::TopFile>, String> {
    Ok(state
        .scanner
        .with_tree(|t| t.top_files.clone())
        .unwrap_or_default())
}

#[tauri::command]
async fn storage_caches(state: State<'_, AppState>) -> Result<Vec<storage::CacheHit>, String> {
    Ok(state
        .scanner
        .with_tree(|t| t.caches.clone())
        .unwrap_or_default())
}

/// Opens File Explorer at a path. Only paths inside the scanned folder are
/// accepted, so the UI can't be used to launch arbitrary things.
#[tauri::command]
async fn storage_reveal(state: State<'_, AppState>, path: String) -> Result<(), String> {
    let allowed = state
        .scanner
        .with_tree(|t| t.contains_path(&path))
        .unwrap_or(false);
    if !allowed || path.contains('"') {
        return Err("Táto cesta nie je v naskenovanom priečinku.".into());
    }
    reveal(&path)
}

#[tauri::command]
async fn storage_owners(state: State<'_, AppState>) -> Result<storage::Owners, String> {
    Ok(state.scanner.with_tree(|t| t.owners()).unwrap_or_default())
}

#[tauri::command]
async fn storage_old(state: State<'_, AppState>) -> Result<Vec<storage::OldFile>, String> {
    Ok(state
        .scanner
        .with_tree(|t| t.old_files.clone())
        .unwrap_or_default())
}

#[tauri::command]
async fn storage_search(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<storage::Hit>, String> {
    Ok(state
        .scanner
        .with_tree(|t| t.search(&query, 200))
        .unwrap_or_default())
}

#[derive(serde::Serialize)]
struct Quick {
    caches: Vec<storage::QuickHit>,
    installers: Vec<storage::OldFile>,
}

/// Sizes the well-known cleanup spots without a full scan (a second or two).
#[tauri::command]
async fn quick_check() -> Quick {
    let downloads = std::env::var("USERPROFILE")
        .ok()
        .map(|u| std::path::PathBuf::from(u).join("Downloads"));
    let (caches, installers) = storage::quick_check(&storage::quick_check_paths(), downloads);
    Quick { caches, installers }
}

#[tauri::command]
async fn system_facts() -> system::Facts {
    system::facts()
}

#[tauri::command]
async fn pc_specs() -> system::Specs {
    system::specs()
}

/// Saves the share card PNG to Pictures\WhatsInside and shows it in Explorer.
#[tauri::command]
async fn save_card(png_base64: String) -> Result<String, String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(png_base64.trim_start_matches("data:image/png;base64,"))
        .map_err(|e| e.to_string())?;
    if !bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        return Err("Neplatný obrázok.".into());
    }
    let home = std::env::var("USERPROFILE")
        .map_err(|_| "Nenašiel som priečinok používateľa.".to_string())?;
    let dir = std::path::Path::new(&home)
        .join("Pictures")
        .join("WhatsInside");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let path = dir.join(format!("WhatsInside-PC-{secs}.png"));
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    let p = path.to_string_lossy().into_owned();
    let _ = reveal(&p);
    Ok(p)
}

#[cfg(windows)]
fn reveal(path: &str) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    let p = std::path::Path::new(path);
    let mut cmd = std::process::Command::new("explorer");
    if p.is_dir() {
        cmd.raw_arg(format!("\"{path}\""));
    } else {
        cmd.raw_arg(format!("/select,\"{path}\""));
    }
    cmd.spawn().map(|_| ()).map_err(|e| e.to_string())
}

#[cfg(not(windows))]
fn reveal(_path: &str) -> Result<(), String> {
    Err("Len na Windows.".into())
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            procs_snapshot,
            autostart_list,
            hw_scan,
            storage_drives,
            storage_start,
            storage_cancel,
            storage_status,
            storage_node,
            storage_top_files,
            storage_caches,
            storage_reveal,
            storage_owners,
            storage_old,
            storage_search,
            quick_check,
            system_facts,
            pc_specs,
            save_card,
        ])
        .run(tauri::generate_context!())
        .expect("WhatsInside sa nepodarilo spustiť");
}
