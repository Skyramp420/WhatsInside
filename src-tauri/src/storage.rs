//! Disk usage: a parallel scan that builds a compact tree, plus the things
//! people actually want to find in it (biggest files, caches that are
//! safe to clear).
//!
//! Read-only. Nothing is deleted from here; the UI can only reveal a path
//! in File Explorer.
//!
//! Not yet: reading the NTFS MFT directly (WizTree speed, needs admin) and
//! hard-link de-duplication (C:\Windows\WinSxS looks bigger than it is).

use rayon::prelude::*;
use serde::Serialize;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Files smaller than this are folded into one "small files" entry per
/// folder. Keeps memory low on disks with millions of tiny files.
const BIG_FILE: u64 = 1 << 20;
const TOP_FILES: usize = 100;
const NONE: u32 = u32::MAX;

// ---------------------------------------------------------------- scan phase

struct ScanDir {
    name: String,
    size: u64,
    files: u64,
    small_size: u64,
    small_count: u64,
    dirs: Vec<ScanDir>,
    big: Vec<(String, u64)>,
}

#[derive(Default)]
pub struct Progress {
    pub files: AtomicU64,
    pub dirs: AtomicU64,
    pub bytes: AtomicU64,
    pub errors: AtomicU64,
    pub cloud_bytes: AtomicU64,
}

struct Ctx {
    progress: Arc<Progress>,
    cancel: Arc<AtomicBool>,
    top: Mutex<BinaryHeap<Reverse<(u64, String)>>>,
    top_min: AtomicU64,
}

impl Ctx {
    fn offer_top(&self, size: u64, path: &Path) {
        if size <= self.top_min.load(Relaxed) {
            return;
        }
        let mut heap = self.top.lock().unwrap();
        heap.push(Reverse((size, path.to_string_lossy().into_owned())));
        if heap.len() > TOP_FILES {
            heap.pop();
        }
        if heap.len() == TOP_FILES {
            if let Some(Reverse((min, _))) = heap.peek() {
                self.top_min.store(*min, Relaxed);
            }
        }
    }
}

/// Size the file actually takes. Cloud placeholders (OneDrive "online
/// only") report their full size but occupy almost nothing on disk.
#[cfg(windows)]
fn disk_len(meta: &std::fs::Metadata, ctx: &Ctx) -> u64 {
    use std::os::windows::fs::MetadataExt;
    const OFFLINE: u32 = 0x1000;
    const RECALL_ON_OPEN: u32 = 0x40000;
    const RECALL_ON_DATA_ACCESS: u32 = 0x40_0000;
    if meta.file_attributes() & (OFFLINE | RECALL_ON_OPEN | RECALL_ON_DATA_ACCESS) != 0 {
        ctx.progress.cloud_bytes.fetch_add(meta.len(), Relaxed);
        0
    } else {
        meta.len()
    }
}

#[cfg(not(windows))]
fn disk_len(meta: &std::fs::Metadata, _ctx: &Ctx) -> u64 {
    meta.len()
}

fn scan_dir(path: &Path, name: String, ctx: &Ctx) -> ScanDir {
    let mut d = ScanDir {
        name,
        size: 0,
        files: 0,
        small_size: 0,
        small_count: 0,
        dirs: Vec::new(),
        big: Vec::new(),
    };
    if ctx.cancel.load(Relaxed) {
        return d;
    }
    let entries = match std::fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => {
            ctx.progress.errors.fetch_add(1, Relaxed);
            return d;
        }
    };
    let mut subdirs: Vec<(PathBuf, String)> = Vec::new();
    for entry in entries.flatten() {
        let Ok(ft) = entry.file_type() else { continue };
        // Junctions and symlinks point elsewhere: counting them would
        // double-count or loop forever.
        if ft.is_symlink() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if ft.is_dir() {
            subdirs.push((entry.path(), name));
        } else if let Ok(meta) = entry.metadata() {
            let len = disk_len(&meta, ctx);
            d.size += len;
            d.files += 1;
            if len >= BIG_FILE {
                ctx.offer_top(len, &entry.path());
                d.big.push((name, len));
            } else {
                d.small_size += len;
                d.small_count += 1;
            }
        }
    }
    ctx.progress.files.fetch_add(d.files, Relaxed);
    ctx.progress.bytes.fetch_add(d.size, Relaxed);
    ctx.progress.dirs.fetch_add(1, Relaxed);

    d.dirs = subdirs
        .into_par_iter()
        .map(|(p, n)| scan_dir(&p, n, ctx))
        .collect();
    for sub in &d.dirs {
        d.size += sub.size;
        d.files += sub.files;
    }
    d
}

// ---------------------------------------------------------------- tree

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Dir,
    File,
    /// All files under 1 MB in one folder, folded together.
    Small,
    /// "Everything else" when a list is cut short (only in views).
    Other,
}

struct Node {
    name: Box<str>,
    size: u64,
    files: u64,
    parent: u32,
    kind: Kind,
    children: Vec<u32>,
}

#[derive(Serialize, Clone, Debug)]
pub struct CacheHit {
    pub label: &'static str,
    pub hint: &'static str,
    pub path: String,
    pub size: u64,
    pub files: u64,
    pub id: u32,
}

#[derive(Serialize, Clone, Debug)]
pub struct TopFile {
    pub path: String,
    pub size: u64,
}

pub struct Tree {
    nodes: Vec<Node>,
    root_path: String,
    pub top_files: Vec<TopFile>,
    pub caches: Vec<CacheHit>,
}

struct CacheRule {
    /// Lower-case path fragment; a leading `=` means "folder named exactly".
    matches: &'static [&'static str],
    label: &'static str,
    hint: &'static str,
}

const CACHE_RULES: &[CacheRule] = &[
    CacheRule {
        matches: &["=cacheclip"],
        label: "DaVinci Resolve – render cache",
        hint: "Resolve si ju vytvorí znova. Zmaž ju v Resolve (Playback → Delete Render Cache) alebo keď Resolve nebeží.",
    },
    CacheRule {
        matches: &["=optimizedmedia"],
        label: "DaVinci Resolve – optimalizované médiá",
        hint: "Nie sú to originály, ale ich znovuvytvorenie trvá dlho. Maž len pri projektoch, na ktorých už nerobíš.",
    },
    CacheRule {
        matches: &["=proxymedia"],
        label: "DaVinci Resolve – proxy",
        hint: "Nie sú to originály. Maž len pri projektoch, na ktorých už nerobíš.",
    },
    CacheRule {
        matches: &[r"\adobe\common\media cache files", r"\adobe\common\media cache", r"\adobe\common\peak files"],
        label: "Adobe – media cache (Premiere, After Effects)",
        hint: "Bezpečné zmazať, keď Adobe appky nebežia. Vytvorí sa znova pri otvorení projektu.",
    },
    CacheRule {
        matches: &[r"\after effects disk cache", r"\adobe\after effects\disk cache", r"\aedisk cache"],
        label: "After Effects – disk cache",
        hint: "Bezpečné zmazať (Edit → Purge → All Memory & Disk Cache).",
    },
    CacheRule {
        matches: &[r"\nvidia\dxcache", r"\nvidia\glcache", r"\nvidia corporation\nv_cache", r"\local\d3dscache", r"\amd\dxcache"],
        label: "Shader cache grafiky",
        hint: "Hry si ju vytvoria znova. Prvé spustenie hry potom môže chvíľu sekať.",
    },
    CacheRule {
        matches: &[r"\steamapps\shadercache"],
        label: "Steam – shader cache",
        hint: "Steam si ju stiahne alebo vytvorí znova.",
    },
    CacheRule {
        matches: &[r"\windows\softwaredistribution\download"],
        label: "Stiahnuté aktualizácie Windowsu",
        hint: "Maž cez Nastavenia → Systém → Úložisko → Dočasné súbory. Windows si ich stiahne znova, ak bude treba.",
    },
    CacheRule {
        matches: &[r"\appdata\local\temp", r"\windows\temp"],
        label: "Dočasné súbory",
        hint: "Bezpečné zmazať. Súbory, ktoré sa práve používajú, Windows zmazať nedovolí, a to je v poriadku.",
    },
    CacheRule {
        matches: &[r"\google\chrome\user data\default\cache", r"\microsoft\edge\user data\default\cache", r"\bravesoftware\brave-browser\user data\default\cache"],
        label: "Cache prehliadača",
        hint: "Bezpečné zmazať v nastaveniach prehliadača. Stránky sa načítajú znova.",
    },
    CacheRule {
        matches: &[r"\discord\cache", r"\discord\code cache"],
        label: "Discord – cache",
        hint: "Bezpečné zmazať, keď Discord nebeží.",
    },
    CacheRule {
        matches: &[r"\$recycle.bin"],
        label: "Kôš",
        hint: "Súbory, ktoré si už zmazal. Vysypaj Kôš.",
    },
];

fn match_cache(lower_path: &str, lower_name: &str) -> Option<&'static CacheRule> {
    CACHE_RULES.iter().find(|r| {
        r.matches.iter().any(|m| match m.strip_prefix('=') {
            Some(exact) => lower_name == exact,
            None => lower_path.ends_with(m),
        })
    })
}

impl Tree {
    fn build(root: ScanDir, root_path: &str, top: Vec<TopFile>) -> Tree {
        let mut t = Tree {
            nodes: Vec::new(),
            root_path: root_path.to_string(),
            top_files: top,
            caches: Vec::new(),
        };
        let root_display = root_path.trim_end_matches(['\\', '/']).to_string();
        t.add_dir(root, NONE, &root_display, false);
        t.caches.sort_by(|a, b| b.size.cmp(&a.size));
        t
    }

    fn add_dir(&mut self, d: ScanDir, parent: u32, path: &str, inside_cache: bool) -> u32 {
        let id = self.nodes.len() as u32;
        let name: Box<str> = if parent == NONE {
            self.root_path.clone().into()
        } else {
            d.name.clone().into()
        };
        self.nodes.push(Node {
            name,
            size: d.size,
            files: d.files,
            parent,
            kind: Kind::Dir,
            children: Vec::new(),
        });

        let mut in_cache = inside_cache;
        if !inside_cache && parent != NONE {
            let lower = path.to_lowercase();
            if let Some(rule) = match_cache(&lower, &d.name.to_lowercase()) {
                if d.size > 0 {
                    self.caches.push(CacheHit {
                        label: rule.label,
                        hint: rule.hint,
                        path: path.to_string(),
                        size: d.size,
                        files: d.files,
                        id,
                    });
                }
                in_cache = true;
            }
        }

        let mut kids = Vec::with_capacity(d.dirs.len() + d.big.len() + 1);
        for sub in d.dirs {
            let child_path = format!("{}\\{}", path, sub.name);
            kids.push(self.add_dir(sub, id, &child_path, in_cache));
        }
        for (name, size) in d.big {
            kids.push(self.push_leaf(name.into(), size, 1, id, Kind::File));
        }
        if d.small_count > 0 {
            let label = format!("{} menších súborov", d.small_count);
            kids.push(self.push_leaf(label.into(), d.small_size, d.small_count, id, Kind::Small));
        }
        kids.sort_by(|a, b| {
            self.nodes[*b as usize]
                .size
                .cmp(&self.nodes[*a as usize].size)
        });
        self.nodes[id as usize].children = kids;
        id
    }

    fn push_leaf(&mut self, name: Box<str>, size: u64, files: u64, parent: u32, kind: Kind) -> u32 {
        let id = self.nodes.len() as u32;
        self.nodes.push(Node {
            name,
            size,
            files,
            parent,
            kind,
            children: Vec::new(),
        });
        id
    }

    pub fn path_of(&self, id: u32) -> String {
        let mut names = Vec::new();
        let mut cur = id;
        while cur != NONE {
            let n = &self.nodes[cur as usize];
            if n.parent == NONE {
                break;
            }
            names.push(n.name.as_ref());
            cur = n.parent;
        }
        let mut p = self.root_path.trim_end_matches(['\\', '/']).to_string();
        if names.is_empty() {
            return self.root_path.clone();
        }
        for n in names.iter().rev() {
            p.push('\\');
            p.push_str(n);
        }
        p
    }

    pub fn contains_path(&self, path: &str) -> bool {
        path.to_lowercase()
            .starts_with(&self.root_path.trim_end_matches(['\\', '/']).to_lowercase())
    }

    fn view(&self, id: u32, depth: u8, limit: usize) -> NodeView {
        let n = &self.nodes[id as usize];
        let children = if depth == 0 || n.kind != Kind::Dir {
            None
        } else {
            let mut v: Vec<NodeView> = n
                .children
                .iter()
                .take(limit)
                .map(|&c| self.view(c, depth - 1, if depth > 1 { 24 } else { limit }))
                .collect();
            if n.children.len() > limit {
                let rest = &n.children[limit..];
                v.push(NodeView {
                    id: None,
                    name: format!("{} ďalších položiek", rest.len()),
                    size: rest.iter().map(|&c| self.nodes[c as usize].size).sum(),
                    files: rest.iter().map(|&c| self.nodes[c as usize].files).sum(),
                    kind: Kind::Other,
                    children: None,
                });
            }
            Some(v)
        };
        NodeView {
            id: Some(id),
            name: n.name.to_string(),
            size: n.size,
            files: n.files,
            kind: n.kind,
            children,
        }
    }

    pub fn describe(&self, id: u32) -> Option<Listing> {
        if id as usize >= self.nodes.len() {
            return None;
        }
        let mut crumbs = Vec::new();
        let mut cur = id;
        while cur != NONE {
            let n = &self.nodes[cur as usize];
            let name = if n.parent == NONE {
                self.root_path.trim_end_matches(['\\', '/']).to_string()
            } else {
                n.name.to_string()
            };
            crumbs.push(Crumb { id: cur, name });
            cur = n.parent;
        }
        crumbs.reverse();
        Some(Listing {
            node: self.view(id, 2, 150),
            path: self.path_of(id),
            crumbs,
        })
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct NodeView {
    pub id: Option<u32>,
    pub name: String,
    pub size: u64,
    pub files: u64,
    pub kind: Kind,
    pub children: Option<Vec<NodeView>>,
}

#[derive(Serialize, Clone, Debug)]
pub struct Crumb {
    pub id: u32,
    pub name: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct Listing {
    pub node: NodeView,
    pub path: String,
    pub crumbs: Vec<Crumb>,
}

// ---------------------------------------------------------------- scanner

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Idle,
    Scanning,
    Done,
    Cancelled,
}

#[derive(Serialize, Clone, Debug)]
pub struct Status {
    pub phase: Phase,
    pub root: Option<String>,
    pub files: u64,
    pub dirs: u64,
    pub bytes: u64,
    pub errors: u64,
    pub cloud_bytes: u64,
    pub elapsed_ms: u64,
}

struct Job {
    root: String,
    started: Instant,
    finished: Option<Instant>,
    progress: Arc<Progress>,
    cancel: Arc<AtomicBool>,
}

#[derive(Default)]
pub struct Scanner {
    job: Mutex<Option<Job>>,
    tree: Arc<Mutex<Option<Tree>>>,
}

impl Scanner {
    pub fn start(&self, root: String) {
        self.cancel();
        *self.tree.lock().unwrap() = None;
        let progress = Arc::new(Progress::default());
        let cancel = Arc::new(AtomicBool::new(false));
        *self.job.lock().unwrap() = Some(Job {
            root: root.clone(),
            started: Instant::now(),
            finished: None,
            progress: progress.clone(),
            cancel: cancel.clone(),
        });
        let tree_slot = self.tree.clone();
        std::thread::spawn(move || {
            let ctx = Ctx {
                progress,
                cancel: cancel.clone(),
                top: Mutex::new(BinaryHeap::new()),
                top_min: AtomicU64::new(0),
            };
            let scanned = scan_dir(Path::new(&root), root.clone(), &ctx);
            if cancel.load(Relaxed) {
                return;
            }
            let mut top: Vec<TopFile> = ctx
                .top
                .into_inner()
                .unwrap()
                .into_iter()
                .map(|Reverse((size, path))| TopFile { path, size })
                .collect();
            top.sort_by(|a, b| b.size.cmp(&a.size));
            let tree = Tree::build(scanned, &root, top);
            *tree_slot.lock().unwrap() = Some(tree);
        });
    }

    pub fn cancel(&self) {
        if let Some(job) = self.job.lock().unwrap().as_ref() {
            job.cancel.store(true, Relaxed);
        }
    }

    pub fn status(&self) -> Status {
        let done = self.tree.lock().unwrap().is_some();
        let mut guard = self.job.lock().unwrap();
        let Some(job) = guard.as_mut() else {
            return Status {
                phase: Phase::Idle,
                root: None,
                files: 0,
                dirs: 0,
                bytes: 0,
                errors: 0,
                cloud_bytes: 0,
                elapsed_ms: 0,
            };
        };
        if done && job.finished.is_none() {
            job.finished = Some(Instant::now());
        }
        let cancelled = job.cancel.load(Relaxed);
        let end = job.finished.unwrap_or_else(Instant::now);
        let p = &job.progress;
        Status {
            phase: if done {
                Phase::Done
            } else if cancelled {
                Phase::Cancelled
            } else {
                Phase::Scanning
            },
            root: Some(job.root.clone()),
            files: p.files.load(Relaxed),
            dirs: p.dirs.load(Relaxed),
            bytes: p.bytes.load(Relaxed),
            errors: p.errors.load(Relaxed),
            cloud_bytes: p.cloud_bytes.load(Relaxed),
            elapsed_ms: end.duration_since(job.started).as_millis() as u64,
        }
    }

    pub fn with_tree<R>(&self, f: impl FnOnce(&Tree) -> R) -> Option<R> {
        self.tree.lock().unwrap().as_ref().map(f)
    }
}

// ---------------------------------------------------------------- drives

#[derive(Serialize, Clone, Debug)]
pub struct Drive {
    pub root: String,
    pub label: String,
    pub fs: String,
    pub total: u64,
    pub free: u64,
    pub removable: bool,
}

#[cfg(windows)]
pub fn drives() -> Vec<Drive> {
    use windows::core::PCWSTR;
    use windows::Win32::Storage::FileSystem::{
        GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDriveStringsW, GetVolumeInformationW,
    };
    const DRIVE_REMOVABLE: u32 = 2;
    const DRIVE_FIXED: u32 = 3;
    let mut out = Vec::new();
    unsafe {
        let mut buf = [0u16; 512];
        let n = GetLogicalDriveStringsW(Some(&mut buf)) as usize;
        for root in buf[..n.min(buf.len())]
            .split(|&c| c == 0)
            .filter(|s| !s.is_empty())
        {
            let mut wroot: Vec<u16> = root.to_vec();
            wroot.push(0);
            let p = PCWSTR(wroot.as_ptr());
            let kind = GetDriveTypeW(p);
            if kind != DRIVE_FIXED && kind != DRIVE_REMOVABLE {
                continue;
            }
            let (mut free, mut total) = (0u64, 0u64);
            if GetDiskFreeSpaceExW(p, None, Some(&mut total), Some(&mut free)).is_err() {
                continue;
            }
            let mut label = [0u16; 261];
            let mut fsname = [0u16; 64];
            let _ = GetVolumeInformationW(p, Some(&mut label), None, None, None, Some(&mut fsname));
            let s = |b: &[u16]| {
                String::from_utf16_lossy(&b[..b.iter().position(|&c| c == 0).unwrap_or(b.len())])
            };
            out.push(Drive {
                root: String::from_utf16_lossy(root),
                label: s(&label),
                fs: s(&fsname),
                total,
                free,
                removable: kind == DRIVE_REMOVABLE,
            });
        }
    }
    out
}

#[cfg(not(windows))]
pub fn drives() -> Vec<Drive> {
    vec![Drive {
        root: "/".into(),
        label: "root".into(),
        fs: String::new(),
        total: 0,
        free: 0,
        removable: false,
    }]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn scans_and_finds_caches() {
        let base = std::env::temp_dir().join(format!("wi-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(base.join("Projekt/CacheClip/sub")).unwrap();
        fs::create_dir_all(base.join("Videá")).unwrap();
        fs::write(
            base.join("Projekt/CacheClip/sub/a.dvcc"),
            vec![0u8; 3 << 20],
        )
        .unwrap();
        fs::write(base.join("Videá/film.mp4"), vec![0u8; 5 << 20]).unwrap();
        fs::write(base.join("Videá/note.txt"), b"hi").unwrap();

        let root = base.to_string_lossy().into_owned();
        let s = Scanner::default();
        s.start(root.clone());
        let t0 = Instant::now();
        while !matches!(s.status().phase, Phase::Done) {
            assert!(t0.elapsed().as_secs() < 10, "scan did not finish");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let st = s.status();
        assert_eq!(st.files, 3);
        assert_eq!(st.bytes, (8 << 20) + 2);

        s.with_tree(|t| {
            assert_eq!(t.top_files[0].size, 5 << 20);
            assert!(t.top_files[0].path.ends_with("film.mp4"));
            assert_eq!(t.caches.len(), 1);
            assert_eq!(t.caches[0].label, "DaVinci Resolve – render cache");
            assert_eq!(t.caches[0].size, 3 << 20);
            let l = t.describe(0).unwrap();
            let kids = l.node.children.unwrap();
            assert_eq!(kids[0].name, "Videá"); // biggest first
            let videa = t.describe(kids[0].id.unwrap()).unwrap();
            let names: Vec<_> = videa
                .node
                .children
                .unwrap()
                .into_iter()
                .map(|c| (c.name, c.kind))
                .collect();
            assert_eq!(
                names,
                vec![
                    ("film.mp4".to_string(), Kind::File),
                    ("1 menších súborov".to_string(), Kind::Small)
                ]
            );
            assert!(videa.path.ends_with("Videá"));
            assert!(t.contains_path(&videa.path));
        })
        .unwrap();
        let _ = fs::remove_dir_all(&base);
    }
}
