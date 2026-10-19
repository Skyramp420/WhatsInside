//! Disk usage: a parallel scan that builds a compact tree, plus the things
//! people actually want to find in it: biggest files, what belongs to which
//! game or program, old files, and caches that are safe to clear.
//!
//! Read-only. Nothing is deleted from here; the UI can only reveal a path
//! in File Explorer.
//!
//! Not yet: reading the NTFS MFT directly (WizTree speed, needs admin) and
//! hard-link de-duplication (C:\Windows\WinSxS looks bigger than it is).

use rayon::prelude::*;
use serde::Serialize;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Files smaller than this are folded into one "small files" entry per
/// folder. Keeps memory low on disks with millions of tiny files.
const BIG_FILE: u64 = 1 << 20;
const TOP_FILES: usize = 100;
const NONE: u32 = u32::MAX;

/// Days since 1970 – enough precision for "how old is this", half the memory of a timestamp.
fn days(t: SystemTime) -> u32 {
    t.duration_since(UNIX_EPOCH)
        .map(|d| (d.as_secs() / 86_400) as u32)
        .unwrap_or(0)
}
pub fn today() -> u32 {
    days(SystemTime::now())
}

// ---------------------------------------------------------------- scan phase

struct ScanFile {
    name: String,
    size: u64,
    mtime: u32,
}

struct ScanDir {
    name: String,
    size: u64,
    files: u64,
    mtime: u32,
    small_size: u64,
    small_count: u64,
    dirs: Vec<ScanDir>,
    big: Vec<ScanFile>,
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
    fn new(progress: Arc<Progress>, cancel: Arc<AtomicBool>) -> Ctx {
        Ctx {
            progress,
            cancel,
            top: Mutex::new(BinaryHeap::new()),
            top_min: AtomicU64::new(0),
        }
    }

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
        mtime: 0,
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
            let mtime = meta.modified().map(days).unwrap_or(0);
            d.size += len;
            d.files += 1;
            d.mtime = d.mtime.max(mtime);
            if len >= BIG_FILE {
                ctx.offer_top(len, &entry.path());
                d.big.push(ScanFile {
                    name,
                    size: len,
                    mtime,
                });
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
        d.mtime = d.mtime.max(sub.mtime);
    }
    d
}

// ---------------------------------------------------------------- labels

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum TagKind {
    Game,
    App,
    AppData,
    System,
    Cache,
}

#[derive(Serialize, Clone, Debug)]
pub struct Tag {
    pub kind: TagKind,
    pub label: String,
    /// Launcher or store, e.g. "Steam".
    pub source: Option<&'static str>,
}

/// Game libraries: (lower-case parent path suffix, store name).
const GAME_LIBS: &[(&str, &str)] = &[
    (r"\steamapps\common", "Steam"),
    (r"\epic games", "Epic Games"),
    (r"\xboxgames", "Xbox"),
    (r"\gog galaxy\games", "GOG"),
    (r"\gog games", "GOG"),
    (r"\riot games", "Riot"),
    (r"\ubisoft game launcher\games", "Ubisoft"),
    (r"\ubisoft\ubisoft game launcher\games", "Ubisoft"),
    (r"\ea games", "EA"),
    (r"\battle.net\games", "Battle.net"),
];

/// Folders inside game libraries that are launchers, not games.
const NOT_GAMES: &[&str] = &[
    "launcher",
    "riot client",
    "directxredist",
    "steamworks shared",
    "epic online services",
];

fn tag_for(parent_lower: &str, name: &str, lower_path: &str, root_lower: &str) -> Option<Tag> {
    let lname = name.to_lowercase();
    for (suffix, store) in GAME_LIBS {
        if parent_lower.ends_with(suffix) {
            if NOT_GAMES.contains(&lname.as_str()) {
                return Some(Tag {
                    kind: TagKind::App,
                    label: name.to_string(),
                    source: Some(store),
                });
            }
            return Some(Tag {
                kind: TagKind::Game,
                label: name.to_string(),
                source: Some(store),
            });
        }
    }
    if parent_lower.ends_with(r"\program files") || parent_lower.ends_with(r"\program files (x86)")
    {
        let kind =
            if lname == "windowsapps" || lname == "common files" || lname.starts_with("windows ") {
                TagKind::System
            } else {
                TagKind::App
            };
        return Some(Tag {
            kind,
            label: name.to_string(),
            source: None,
        });
    }
    if (parent_lower.ends_with(r"\appdata\local")
        || parent_lower.ends_with(r"\appdata\roaming")
        || parent_lower.ends_with(r"\appdata\locallow"))
        && lname != "temp"
    {
        return Some(Tag {
            kind: TagKind::AppData,
            label: name.to_string(),
            source: None,
        });
    }
    let rel = lower_path.strip_prefix(root_lower).unwrap_or(lower_path);
    match rel {
        r"\windows" => Some(Tag {
            kind: TagKind::System,
            label: "Windows".into(),
            source: None,
        }),
        r"\programdata" => Some(Tag {
            kind: TagKind::AppData,
            label: "Dáta programov".into(),
            source: None,
        }),
        _ => None,
    }
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
    mtime: u32,
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

#[derive(Serialize, Clone, Debug)]
pub struct Owned {
    pub id: u32,
    pub label: String,
    pub source: Option<&'static str>,
    pub path: String,
    pub size: u64,
    /// Days since anything inside changed.
    pub age_days: u32,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct Owners {
    pub games: Vec<Owned>,
    pub apps: Vec<Owned>,
    pub games_total: u64,
}

#[derive(Serialize, Clone, Debug)]
pub struct OldFile {
    pub id: u32,
    pub path: String,
    pub size: u64,
    pub age_days: u32,
    /// "installer" for setup files and archives in Downloads, "old" otherwise.
    pub reason: &'static str,
}

#[derive(Serialize, Clone, Debug)]
pub struct Hit {
    pub id: u32,
    pub parent: u32,
    pub name: String,
    pub path: String,
    pub size: u64,
    pub kind: Kind,
    pub age_days: u32,
    pub owner: Option<String>,
}

pub struct Tree {
    nodes: Vec<Node>,
    root_path: String,
    tags: HashMap<u32, Tag>,
    today: u32,
    pub top_files: Vec<TopFile>,
    pub caches: Vec<CacheHit>,
    pub old_files: Vec<OldFile>,
}

pub struct CacheRule {
    /// Lower-case path fragment; a leading `=` means "folder named exactly".
    matches: &'static [&'static str],
    pub label: &'static str,
    pub hint: &'static str,
}

pub const CACHE_RULES: &[CacheRule] = &[
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
        matches: &[r"\steamapps\downloading"],
        label: "Steam – nedokončené sťahovanie",
        hint: "Zvyšky prerušených aktualizácií. Ak nič nesťahuješ, dá sa zmazať (Steam → Nastavenia → Sťahovanie → Vymazať cache).",
    },
    CacheRule {
        matches: &[r"\windows\softwaredistribution\download"],
        label: "Stiahnuté aktualizácie Windowsu",
        hint: "Maž cez Nastavenia → Systém → Úložisko → Dočasné súbory. Windows si ich stiahne znova, ak bude treba.",
    },
    CacheRule {
        matches: &[r"\windows.old", r"\$windows.~bt", r"\$windows.~ws"],
        label: "Stará verzia Windowsu",
        hint: "Zostala po veľkej aktualizácii. Maž cez Nastavenia → Systém → Úložisko → Dočasné súbory → Predchádzajúce inštalácie Windowsu.",
    },
    CacheRule {
        matches: &[r"\windows\minidump", r"\windows\livekernelreports", r"\appdata\local\crashdumps"],
        label: "Výpisy po pádoch",
        hint: "Záznamy po pádoch appiek a systému. Treba ich len pri hľadaní príčiny pádu, inak sa dajú zmazať.",
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

const INSTALLER_EXT: &[&str] = &["exe", "msi", "msix", "iso", "zip", "rar", "7z"];
/// Where old files are expected and not worth flagging.
const OLD_SKIP: &[&str] = &[
    r"\windows\",
    r"\program files",
    r"\programdata\",
    r"\appdata\",
    r"\steamapps\",
    r"\xboxgames\",
    r"\epic games\",
    r"\$recycle.bin",
    r"\windowsapps\",
    r"\riot games\",
];

fn is_installer(lower_name: &str) -> bool {
    lower_name
        .rsplit_once('.')
        .is_some_and(|(_, ext)| INSTALLER_EXT.contains(&ext))
}

struct Build<'a> {
    t: &'a mut Tree,
    root_lower: String,
    old: Vec<OldFile>,
}

impl Tree {
    fn build(root: ScanDir, root_path: &str, top: Vec<TopFile>, today: u32) -> Tree {
        let mut t = Tree {
            nodes: Vec::new(),
            root_path: root_path.to_string(),
            tags: HashMap::new(),
            today,
            top_files: top,
            caches: Vec::new(),
            old_files: Vec::new(),
        };
        let root_display = root_path.trim_end_matches(['\\', '/']).to_string();
        let mut b = Build {
            root_lower: root_display.to_lowercase(),
            t: &mut t,
            old: Vec::new(),
        };
        b.add_dir(root, NONE, &root_display, "", false);
        let mut old = std::mem::take(&mut b.old);
        old.sort_by_key(|o| Reverse(o.size));
        old.truncate(150);
        t.old_files = old;
        t.caches.sort_by_key(|c| Reverse(c.size));
        t
    }

    fn push_leaf(
        &mut self,
        name: Box<str>,
        size: u64,
        files: u64,
        mtime: u32,
        parent: u32,
        kind: Kind,
    ) -> u32 {
        let id = self.nodes.len() as u32;
        self.nodes.push(Node {
            name,
            size,
            files,
            mtime,
            parent,
            kind,
            children: Vec::new(),
        });
        id
    }

    fn age(&self, mtime: u32) -> u32 {
        if mtime == 0 {
            0
        } else {
            self.today.saturating_sub(mtime)
        }
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
        if names.is_empty() {
            return self.root_path.clone();
        }
        let mut p = self.root_path.trim_end_matches(['\\', '/']).to_string();
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

    /// Nearest labelled ancestor (or the node itself).
    fn owner(&self, id: u32) -> Option<&Tag> {
        let mut cur = id;
        while cur != NONE {
            if let Some(t) = self.tags.get(&cur) {
                return Some(t);
            }
            cur = self.nodes[cur as usize].parent;
        }
        None
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
                    age_days: 0,
                    tag: None,
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
            age_days: self.age(n.mtime),
            tag: self.tags.get(&id).cloned(),
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
            owner: self.owner(id).cloned(),
        })
    }

    pub fn owners(&self) -> Owners {
        let mut o = Owners::default();
        for (&id, tag) in &self.tags {
            let n = &self.nodes[id as usize];
            if n.size == 0 {
                continue;
            }
            let item = Owned {
                id,
                label: tag.label.clone(),
                source: tag.source,
                path: self.path_of(id),
                size: n.size,
                age_days: self.age(n.mtime),
            };
            match tag.kind {
                TagKind::Game => o.games.push(item),
                TagKind::App => o.apps.push(item),
                _ => {}
            }
        }
        o.games.sort_by_key(|g| Reverse(g.size));
        o.apps.sort_by_key(|a| Reverse(a.size));
        o.apps.truncate(60);
        o.games_total = o.games.iter().map(|g| g.size).sum();
        o
    }

    /// Case-insensitive search in folder and file names. Every word of the
    /// query has to appear. Files under 1 MB are not kept, so not found.
    pub fn search(&self, query: &str, limit: usize) -> Vec<Hit> {
        let words: Vec<String> = query
            .to_lowercase()
            .split_whitespace()
            .map(str::to_string)
            .collect();
        if words.is_empty() {
            return Vec::new();
        }
        let mut hits: Vec<u32> = self
            .nodes
            .par_iter()
            .enumerate()
            .filter(|(_, n)| matches!(n.kind, Kind::Dir | Kind::File) && n.parent != NONE)
            .filter(|(_, n)| {
                let name = n.name.to_lowercase();
                words.iter().all(|w| name.contains(w.as_str()))
            })
            .map(|(i, _)| i as u32)
            .collect();
        hits.sort_by_key(|&i| Reverse(self.nodes[i as usize].size));
        hits.truncate(limit);
        hits.into_iter()
            .map(|id| {
                let n = &self.nodes[id as usize];
                Hit {
                    id,
                    parent: n.parent,
                    name: n.name.to_string(),
                    path: self.path_of(id),
                    size: n.size,
                    kind: n.kind,
                    age_days: self.age(n.mtime),
                    owner: self.owner(id).map(|t| t.label.clone()),
                }
            })
            .collect()
    }
}

impl Build<'_> {
    fn add_dir(
        &mut self,
        d: ScanDir,
        parent: u32,
        path: &str,
        parent_lower: &str,
        inside_cache: bool,
    ) -> u32 {
        let t = &mut *self.t;
        let id = t.nodes.len() as u32;
        let name: Box<str> = if parent == NONE {
            t.root_path.clone().into()
        } else {
            d.name.clone().into()
        };
        t.nodes.push(Node {
            name,
            size: d.size,
            files: d.files,
            mtime: d.mtime,
            parent,
            kind: Kind::Dir,
            children: Vec::new(),
        });

        let lower = path.to_lowercase();
        let mut in_cache = inside_cache;
        if parent != NONE {
            if !inside_cache {
                if let Some(rule) = match_cache(&lower, &d.name.to_lowercase()) {
                    if d.size > 0 {
                        t.caches.push(CacheHit {
                            label: rule.label,
                            hint: rule.hint,
                            path: path.to_string(),
                            size: d.size,
                            files: d.files,
                            id,
                        });
                        t.tags.insert(
                            id,
                            Tag {
                                kind: TagKind::Cache,
                                label: rule.label.to_string(),
                                source: None,
                            },
                        );
                    }
                    in_cache = true;
                }
            }
            if !t.tags.contains_key(&id) {
                if let Some(tag) = tag_for(parent_lower, &d.name, &lower, &self.root_lower) {
                    t.tags.insert(id, tag);
                }
            }
        }

        let today = t.today;
        let in_downloads = lower.contains(r"\downloads");
        let with_sep = format!("{lower}\\");
        let skip_old = OLD_SKIP.iter().any(|s| with_sep.contains(s));
        let mut kids = Vec::with_capacity(d.dirs.len() + d.big.len() + 1);
        for sub in d.dirs {
            let child_path = format!("{}\\{}", path, sub.name);
            kids.push(self.add_dir(sub, id, &child_path, &lower, in_cache));
        }
        let t = &mut *self.t;
        for f in d.big {
            let age = if f.mtime == 0 {
                0
            } else {
                today.saturating_sub(f.mtime)
            };
            let lname = f.name.to_lowercase();
            let reason = if in_downloads && age >= 30 && is_installer(&lname) {
                Some("installer")
            } else if !skip_old && !in_cache && age >= 365 && f.size >= 50 << 20 {
                Some("old")
            } else {
                None
            };
            let fid = t.push_leaf(f.name.clone().into(), f.size, 1, f.mtime, id, Kind::File);
            if let Some(reason) = reason {
                self.old.push(OldFile {
                    id: fid,
                    path: format!("{}\\{}", path, f.name),
                    size: f.size,
                    age_days: age,
                    reason,
                });
            }
            kids.push(fid);
        }
        let t = &mut *self.t;
        if d.small_count > 0 {
            let label = format!("{} menších súborov", d.small_count);
            kids.push(t.push_leaf(
                label.into(),
                d.small_size,
                d.small_count,
                0,
                id,
                Kind::Small,
            ));
        }
        kids.sort_by_key(|k| Reverse(t.nodes[*k as usize].size));
        t.nodes[id as usize].children = kids;
        id
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct NodeView {
    pub id: Option<u32>,
    pub name: String,
    pub size: u64,
    pub files: u64,
    pub kind: Kind,
    pub age_days: u32,
    pub tag: Option<Tag>,
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
    /// Game/program this folder belongs to, if any.
    pub owner: Option<Tag>,
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
            let ctx = Ctx::new(progress, cancel.clone());
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
            top.sort_by_key(|f| Reverse(f.size));
            let tree = Tree::build(scanned, &root, top, today());
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

// ---------------------------------------------------------------- quick check

/// One known cleanup spot, sized without a full scan.
#[derive(Serialize, Clone, Debug)]
pub struct QuickHit {
    pub label: &'static str,
    pub hint: &'static str,
    pub path: String,
    pub size: u64,
    pub files: u64,
}

/// Sizes a short list of known cache folders (and old installers in
/// Downloads) in a second or two, so the helper has numbers even before a
/// full disk scan.
pub fn quick_check(
    paths: &[(String, &'static CacheRule)],
    downloads: Option<PathBuf>,
) -> (Vec<QuickHit>, Vec<OldFile>) {
    let mut hits: Vec<QuickHit> = paths
        .par_iter()
        .filter(|(p, _)| Path::new(p).is_dir())
        .map(|(p, rule)| {
            let ctx = Ctx::new(
                Arc::new(Progress::default()),
                Arc::new(AtomicBool::new(false)),
            );
            let d = scan_dir(Path::new(p), String::new(), &ctx);
            QuickHit {
                label: rule.label,
                hint: rule.hint,
                path: p.clone(),
                size: d.size,
                files: d.files,
            }
        })
        .filter(|h| h.size > 0)
        .collect();
    // Merge entries that share a label (e.g. two Temp folders).
    hits.sort_by(|a, b| a.label.cmp(b.label));
    let mut merged: Vec<QuickHit> = Vec::new();
    for h in hits {
        match merged.last_mut() {
            Some(m) if m.label == h.label => {
                m.size += h.size;
                m.files += h.files;
            }
            _ => merged.push(h),
        }
    }
    merged.sort_by_key(|h| Reverse(h.size));

    let mut installers = Vec::new();
    if let Some(dl) = downloads {
        let now = today();
        if let Ok(entries) = std::fs::read_dir(&dl) {
            for e in entries.flatten() {
                let Ok(meta) = e.metadata() else { continue };
                if !meta.is_file() {
                    continue;
                }
                let name = e.file_name().to_string_lossy().to_lowercase();
                let age = meta
                    .modified()
                    .map(|m| now.saturating_sub(days(m)))
                    .unwrap_or(0);
                if age >= 30 && is_installer(&name) {
                    installers.push(OldFile {
                        id: NONE,
                        path: e.path().to_string_lossy().into_owned(),
                        size: meta.len(),
                        age_days: age,
                        reason: "installer",
                    });
                }
            }
        }
        installers.sort_by_key(|o| Reverse(o.size));
    }
    (merged, installers)
}

/// The well-known spots for this machine.
pub fn quick_check_paths() -> Vec<(String, &'static CacheRule)> {
    let env = |k: &str| std::env::var(k).ok();
    let win = env("SystemRoot").unwrap_or_else(|| r"C:\Windows".into());
    let sysdrive = env("SystemDrive").unwrap_or_else(|| "C:".into());
    let rule = |label: &str| {
        CACHE_RULES
            .iter()
            .find(|r| r.label == label)
            .expect("cache rule")
    };
    let mut out = Vec::new();
    let mut add = |p: Option<String>, label: &str| {
        if let Some(p) = p {
            out.push((p, rule(label)));
        }
    };
    add(env("TEMP"), "Dočasné súbory");
    add(Some(format!(r"{win}\Temp")), "Dočasné súbory");
    add(
        Some(format!(r"{win}\SoftwareDistribution\Download")),
        "Stiahnuté aktualizácie Windowsu",
    );
    add(
        env("LOCALAPPDATA").map(|l| format!(r"{l}\NVIDIA\DXCache")),
        "Shader cache grafiky",
    );
    add(
        env("LOCALAPPDATA").map(|l| format!(r"{l}\NVIDIA\GLCache")),
        "Shader cache grafiky",
    );
    add(
        env("LOCALAPPDATA").map(|l| format!(r"{l}\D3DSCache")),
        "Shader cache grafiky",
    );
    add(
        env("LOCALAPPDATA").map(|l| format!(r"{l}\AMD\DxCache")),
        "Shader cache grafiky",
    );
    add(
        env("APPDATA").map(|a| format!(r"{a}\Adobe\Common\Media Cache Files")),
        "Adobe – media cache (Premiere, After Effects)",
    );
    add(
        env("APPDATA").map(|a| format!(r"{a}\Adobe\Common\Media Cache")),
        "Adobe – media cache (Premiere, After Effects)",
    );
    add(
        env("LOCALAPPDATA").map(|l| format!(r"{l}\CrashDumps")),
        "Výpisy po pádoch",
    );
    add(Some(format!(r"{win}\Minidump")), "Výpisy po pádoch");
    add(
        Some(format!(r"{win}\LiveKernelReports")),
        "Výpisy po pádoch",
    );
    add(
        Some(format!(r"{sysdrive}\Windows.old")),
        "Stará verzia Windowsu",
    );
    add(
        env("APPDATA").map(|a| format!(r"{a}\discord\Cache")),
        "Discord – cache",
    );
    add(
        env("LOCALAPPDATA").map(|l| format!(r"{l}\Google\Chrome\User Data\Default\Cache")),
        "Cache prehliadača",
    );
    add(
        env("LOCALAPPDATA").map(|l| format!(r"{l}\Microsoft\Edge\User Data\Default\Cache")),
        "Cache prehliadača",
    );
    out
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

    fn wait(s: &Scanner) {
        let t0 = Instant::now();
        while !matches!(s.status().phase, Phase::Done) {
            assert!(t0.elapsed().as_secs() < 10, "scan did not finish");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    #[test]
    fn scans_labels_and_searches() {
        let base = std::env::temp_dir().join(format!("wi-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(base.join("Projekt/CacheClip/sub")).unwrap();
        fs::create_dir_all(base.join("Videá")).unwrap();
        fs::create_dir_all(base.join("SteamLibrary/steamapps/common/Hollow Knight")).unwrap();
        fs::create_dir_all(base.join("Program Files/DaVinci Resolve")).unwrap();
        fs::write(
            base.join("Projekt/CacheClip/sub/a.dvcc"),
            vec![0u8; 3 << 20],
        )
        .unwrap();
        fs::write(base.join("Videá/film.mp4"), vec![0u8; 5 << 20]).unwrap();
        fs::write(base.join("Videá/note.txt"), b"hi").unwrap();
        fs::write(
            base.join("SteamLibrary/steamapps/common/Hollow Knight/data.bin"),
            vec![0u8; 2 << 20],
        )
        .unwrap();
        fs::write(
            base.join("Program Files/DaVinci Resolve/Resolve.exe"),
            vec![0u8; 1 << 20],
        )
        .unwrap();

        let s = Scanner::default();
        s.start(base.to_string_lossy().into_owned());
        wait(&s);
        assert_eq!(s.status().files, 5);

        s.with_tree(|t| {
            assert!(t.top_files[0].path.ends_with("film.mp4"));
            assert_eq!(t.caches.len(), 1);
            assert_eq!(t.caches[0].label, "DaVinci Resolve – render cache");

            let o = t.owners();
            assert_eq!(o.games.len(), 1);
            assert_eq!(o.games[0].label, "Hollow Knight");
            assert_eq!(o.games[0].source, Some("Steam"));
            assert_eq!(o.games[0].size, 2 << 20);
            assert_eq!(o.apps[0].label, "DaVinci Resolve");

            let l = t.describe(0).unwrap();
            let kids = l.node.children.unwrap();
            assert_eq!(kids[0].name, "Videá");
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

            let hits = t.search("hollow", 10);
            assert_eq!(hits.len(), 1);
            assert_eq!(hits[0].kind, Kind::Dir);
            let hits = t.search("data", 10);
            assert_eq!(hits[0].name, "data.bin");
            assert_eq!(hits[0].owner.as_deref(), Some("Hollow Knight"));
            assert!(t.search("  ", 10).is_empty());
        })
        .unwrap();
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn tags_game_libraries_and_programs() {
        let root = r"c:";
        let t = |parent: &str, name: &str| {
            tag_for(
                &parent.to_lowercase(),
                name,
                &format!("{}\\{}", parent.to_lowercase(), name.to_lowercase()),
                root,
            )
        };
        assert_eq!(
            t(r"C:\Program Files\Epic Games", "Fortnite")
                .unwrap()
                .source,
            Some("Epic Games")
        );
        assert_eq!(
            t(r"C:\Program Files\Epic Games", "Launcher").unwrap().kind,
            TagKind::App
        );
        assert_eq!(
            t(r"D:\XboxGames", "Forza Horizon 5").unwrap().kind,
            TagKind::Game
        );
        assert_eq!(
            t(r"C:\Program Files", "Blender Foundation").unwrap().kind,
            TagKind::App
        );
        assert_eq!(
            t(r"C:\Program Files", "WindowsApps").unwrap().kind,
            TagKind::System
        );
        assert_eq!(
            t(r"C:\Users\Alex\AppData\Roaming", "discord").unwrap().kind,
            TagKind::AppData
        );
        assert!(t(r"C:\Users\Alex\AppData\Local", "Temp").is_none());
        assert_eq!(t(r"C:", "Windows").unwrap().kind, TagKind::System);
        assert!(t(r"C:\Users", "Alex").is_none());
    }

    #[test]
    fn spots_installers_and_cleanup_rules() {
        assert!(is_installer("setup_obs_30.exe"));
        assert!(is_installer("win11.iso"));
        assert!(!is_installer("projekt.drp"));
        assert_eq!(
            match_cache(r"c:\windows.old", "windows.old").unwrap().label,
            "Stará verzia Windowsu"
        );
        assert_eq!(
            match_cache(r"c:\users\a\appdata\local\crashdumps", "crashdumps")
                .unwrap()
                .label,
            "Výpisy po pádoch"
        );
        assert!(match_cache(r"c:\users\a\videos", "videos").is_none());
        // Every quick-check label must exist as a rule (expect() would panic otherwise).
        assert!(!quick_check_paths().is_empty());
    }

    #[test]
    fn quick_check_sizes_folders() {
        let base = std::env::temp_dir().join(format!("wi-qc-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(base.join("tmp1")).unwrap();
        fs::create_dir_all(base.join("tmp2")).unwrap();
        fs::write(base.join("tmp1/a"), vec![0u8; 1000]).unwrap();
        fs::write(base.join("tmp2/b"), vec![0u8; 500]).unwrap();
        let rule = CACHE_RULES
            .iter()
            .find(|r| r.label == "Dočasné súbory")
            .unwrap();
        let paths = vec![
            (base.join("tmp1").to_string_lossy().into_owned(), rule),
            (base.join("tmp2").to_string_lossy().into_owned(), rule),
            (base.join("missing").to_string_lossy().into_owned(), rule),
        ];
        let (hits, inst) = quick_check(&paths, Some(base.clone()));
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].size, 1500);
        assert_eq!(hits[0].files, 2);
        assert!(inst.is_empty());
        let _ = fs::remove_dir_all(&base);
    }
}
