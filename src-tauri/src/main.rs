//! WhatsInside – ľahký prehľad o tom, čo je v tvojom PC. Od WheresTheApp.
//!
//! Changes happen only when the user asks, are reversible and logged (see
//! `actions`). When the window is closed nothing keeps running: no service,
//! no tray icon, no background thread.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![cfg_attr(not(windows), allow(dead_code))]

mod actions;
mod appx;
mod autostart;
mod bench;
mod catalog;
mod display;
mod games;
mod gpu;
mod hw;
mod known;
mod procs;
mod storage;
mod system;
mod tweaks;
mod update;

use std::sync::Mutex;
use tauri::State;

#[derive(Default)]
struct AppState {
    sampler: Mutex<procs::Sampler>,
    scanner: storage::Scanner,
    bench: bench::Bench,
}

#[tauri::command]
async fn procs_snapshot(state: State<'_, AppState>) -> Result<procs::Snapshot, String> {
    Ok(state.sampler.lock().map_err(|e| e.to_string())?.snapshot())
}

#[tauri::command]
async fn autostart_list() -> Vec<autostart::Item> {
    autostart::list(&actions::changed_services())
}

#[tauri::command]
async fn actions_apply(actions: Vec<actions::Action>) -> Vec<actions::Outcome> {
    actions::apply(actions)
}

#[tauri::command]
async fn changes_list() -> Vec<actions::Change> {
    actions::changes()
}

#[tauri::command]
async fn changes_undo(index: usize) -> Result<(), String> {
    actions::undo(index)
}

#[tauri::command]
async fn tweaks_list() -> Vec<tweaks::State> {
    tweaks::list()
}

#[tauri::command]
async fn displays_list() -> Vec<display::Display> {
    display::list()
}

#[tauri::command]
async fn bench_start(
    state: State<'_, AppState>,
    exe: String,
    label: String,
    seconds: u32,
    delay: u32,
) -> Result<(), String> {
    state.bench.start(exe, label, seconds, delay)
}

#[tauri::command]
async fn bench_status(state: State<'_, AppState>) -> Result<bench::Status, String> {
    Ok(state.bench.status())
}

#[tauri::command]
async fn bench_cancel(state: State<'_, AppState>) -> Result<(), String> {
    state.bench.cancel()
}

#[tauri::command]
async fn bench_list() -> Vec<bench::Run> {
    bench::list()
}

#[tauri::command]
async fn bench_delete(id: u64) {
    bench::delete(id)
}

#[tauri::command]
async fn gpu_sample() -> Option<gpu::Sample> {
    gpu::sample()
}

#[tauri::command]
async fn appx_list() -> Vec<appx::App> {
    appx::list()
}

#[tauri::command]
async fn games_list() -> Vec<games::Game> {
    games::list()
}

#[tauri::command]
async fn shader_caches() -> Vec<games::ShaderCache> {
    games::shader_caches()
}

#[tauri::command]
async fn update_check() -> update::Check {
    update::check()
}

/// Downloads the new version, starts the installer and quits.
#[tauri::command]
async fn update_install(app: tauri::AppHandle, latest: update::Latest) -> Result<(), String> {
    update::install(&latest)?;
    app.exit(0);
    Ok(())
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
    // Elevated helper mode: run a batch of changes after the UAC prompt, then quit.
    let args: Vec<String> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--apply") {
        if let (Some(input), Some(output)) = (args.get(i + 1), args.get(i + 2)) {
            actions::run_batch(input, output);
        }
        return;
    }
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
            actions_apply,
            changes_list,
            changes_undo,
            update_check,
            update_install,
            tweaks_list,
            displays_list,
            bench_start,
            bench_status,
            bench_cancel,
            bench_list,
            bench_delete,
            gpu_sample,
            games_list,
            appx_list,
            shader_caches,
        ])
        .run(tauri::generate_context!())
        .expect("WhatsInside sa nepodarilo spustiť");
}
