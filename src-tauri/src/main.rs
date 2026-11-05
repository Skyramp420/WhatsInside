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
mod deep;
mod display;
mod games;
mod gpu;
mod hw;
mod known;
mod mft;
mod procs;
mod regw;
mod stability;
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
async fn actions_apply(actions: Vec<actions::Action>, restore: Option<bool>) -> Vec<actions::Outcome> {
    actions::apply_with(actions, restore.unwrap_or(false))
}

#[tauri::command]
async fn stability_report() -> stability::Report {
    stability::report()
}

#[tauri::command]
async fn disks_list() -> Vec<deep::Disk> {
    deep::disks()
}

/// SSD wear, boot times and restore points: one elevated run.
#[tauri::command]
async fn deep_check() -> Result<deep::Deep, String> {
    let out = std::env::temp_dir().join(format!("whatsinside-deep-{}.json", std::process::id()));
    let code = actions::run_self_elevated(&format!("--deep \"{}\"", out.to_string_lossy()))?;
    let text = std::fs::read_to_string(&out).unwrap_or_default();
    let _ = std::fs::remove_file(&out);
    if code != 0 {
        return Err(format!("Hĺbková kontrola skončila s chybou {code}."));
    }
    deep::parse_deep(&text).ok_or_else(|| "Hĺbková kontrola nevrátila výsledok.".to_string())
}

#[derive(serde::Serialize)]
struct RestoreInfo {
    protection: bool,
    last: u64,
    error: Option<String>,
}

#[tauri::command]
async fn restore_status() -> RestoreInfo {
    let s = actions::restore_state();
    RestoreInfo { protection: deep::protection(), last: s.last, error: s.error }
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
async fn storage_start(
    state: State<'_, AppState>,
    root: String,
    fast: Option<bool>,
) -> Result<(), String> {
    if !std::path::Path::new(&root).is_dir() {
        return Err("Tento priečinok neexistuje.".into());
    }
    let letter = drive_letter(&root);
    match (fast.unwrap_or(false), letter) {
        (true, Some(l)) => state.scanner.start_with(root, Some(Box::new(move || fast_scan(l)))),
        _ => state.scanner.start(root),
    }
    Ok(())
}

/// "C:\\" → 'C'. Only whole drives can be read through the MFT.
fn drive_letter(root: &str) -> Option<char> {
    let r = root.trim_end_matches(['\\', '/']);
    let mut c = r.chars();
    match (c.next(), c.next(), c.next()) {
        (Some(l), Some(':'), None) if l.is_ascii_alphabetic() => Some(l.to_ascii_uppercase()),
        _ => None,
    }
}

/// Reads a drive's MFT: directly when we're admin, otherwise through an
/// elevated copy of ourselves (one UAC prompt).
fn fast_scan(letter: char) -> Result<mft::Volume, String> {
    if actions::is_admin() {
        return read_mft(letter);
    }
    let out = std::env::temp_dir().join(format!("whatsinside-mft-{}-{}.bin", std::process::id(), letter));
    let code = actions::run_self_elevated(&format!("--mft {letter} \"{}\"", out.to_string_lossy()))?;
    let bytes = std::fs::read(&out);
    let err = std::fs::read_to_string(out.with_extension("err")).ok();
    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(out.with_extension("err"));
    if code != 0 {
        return Err(err.unwrap_or_else(|| format!("Rýchly scan skončil s chybou {code}.")));
    }
    mft::read_file(&bytes.map_err(|e| e.to_string())?)
}

fn read_mft(letter: char) -> Result<mft::Volume, String> {
    let mut dev = std::fs::File::open(format!("\\\\.\\{letter}:"))
        .map_err(|e| format!("Disk sa nedá otvoriť: {e}"))?;
    // Raw volume reads must stay sector-aligned, so no BufReader here:
    // every read below is a whole sector, record or cluster run.
    mft::read_volume(&mut dev, &|_| {})
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
async fn storage_growth(state: State<'_, AppState>) -> Result<storage::Growth, String> {
    Ok(state.scanner.with_tree(|t| t.growth()).unwrap_or_default())
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

#[derive(serde::Deserialize)]
struct CleanItem {
    /// "cache" (delete contents), "trash" (to the Recycle Bin), "bin" (empty it)
    kind: String,
    path: String,
    label: String,
}

/// Cleans caches, sends files to the Recycle Bin or empties it. Cache
/// folders are checked against the known rules, files must be inside the
/// scanned folder. Afterwards the scan result follows along.
#[tauri::command]
async fn storage_clean(state: State<'_, AppState>, items: Vec<CleanItem>) -> Result<Vec<actions::Outcome>, String> {
    let mut acts = Vec::new();
    for it in &items {
        acts.push(match it.kind.as_str() {
            "cache" => {
                if storage::clean_rule(&it.path).is_none() {
                    return Err(format!("{} nie je známa cache.", it.path));
                }
                actions::Action::Clean { path: it.path.clone(), label: it.label.clone() }
            }
            "trash" => {
                let inside = state.scanner.with_tree(|t| t.contains_path(&it.path)).unwrap_or(false);
                if !inside {
                    return Err("Tento súbor nie je v naskenovanom priečinku.".into());
                }
                actions::Action::Trash { path: it.path.clone(), label: it.label.clone() }
            }
            "bin" => actions::Action::EmptyBin,
            _ => return Err("Neznáma akcia.".into()),
        });
    }
    let res = actions::apply(acts);
    state.scanner.with_tree_mut(|t| {
        for (it, r) in items.iter().zip(&res) {
            if let (true, Some(f)) = (r.ok, r.freed) {
                if it.kind != "bin" {
                    t.shrink(&it.path, f, it.kind == "trash");
                }
            }
        }
    });
    Ok(res)
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
    if let Some(i) = args.iter().position(|a| a == "--mft") {
        // Elevated helper: read the drive's MFT into a compact file.
        let (Some(l), Some(out)) = (args.get(i + 1).and_then(|s| s.chars().next()), args.get(i + 2)) else {
            std::process::exit(2);
        };
        let out = std::path::PathBuf::from(out);
        match read_mft(l.to_ascii_uppercase()) {
            Ok(v) => {
                let ok = std::fs::File::create(&out)
                    .map(std::io::BufWriter::new)
                    .and_then(|mut w| {
                        mft::write_volume(&v, &mut w)?;
                        std::io::Write::flush(&mut w)
                    });
                std::process::exit(if ok.is_ok() { 0 } else { 3 });
            }
            Err(e) => {
                let _ = std::fs::write(out.with_extension("err"), e);
                std::process::exit(1);
            }
        }
    }
    if let Some(i) = args.iter().position(|a| a == "--deep") {
        let code = args.get(i + 1).map(|o| deep::run_helper(o)).unwrap_or(2);
        std::process::exit(code);
    }
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
            storage_clean,
            storage_growth,
            stability_report,
            disks_list,
            deep_check,
            restore_status,
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

#[cfg(test)]
mod tests {
    #[test]
    fn only_whole_drives_go_fast() {
        assert_eq!(super::drive_letter("C:\\"), Some('C'));
        assert_eq!(super::drive_letter("d:"), Some('D'));
        assert_eq!(super::drive_letter("C:\\Users"), None);
    }
}
