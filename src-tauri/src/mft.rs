//! Fast NTFS scan: read the Master File Table directly, like WizTree does.
//!
//! Instead of asking Windows about every folder (slow: millions of calls),
//! we read the MFT in big sequential chunks and parse the file records
//! ourselves. A whole drive takes seconds. Reading a raw volume needs admin
//! rights, so the app runs this in an elevated copy of itself
//! (`--mft C: out.bin`) and reads the result.
//!
//! The parser works on anything `Read + Seek`, so it is tested on a small
//! synthetic NTFS image.

use std::io::{Read, Seek, SeekFrom, Write};

/// One file or folder from the MFT.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub parent: u32,
    pub name: String,
    pub size: u64,
    /// Days since 1970.
    pub mtime: u32,
    pub dir: bool,
    /// OneDrive "online only" and similar: listed size isn't on the disk.
    pub cloud: bool,
}

/// Entries indexed by MFT record number.
#[derive(Debug, Default)]
pub struct Volume {
    pub entries: Vec<Option<Entry>>,
}

pub const ROOT: u32 = 5;

#[derive(Debug, Clone, Copy)]
pub struct Boot {
    pub bytes_per_sector: u32,
    pub cluster: u64,
    pub mft_lcn: u64,
    pub record: usize,
}

pub fn parse_boot(b: &[u8]) -> Result<Boot, String> {
    if b.len() < 512 || &b[3..11] != b"NTFS    " {
        return Err("Disk nie je NTFS.".into());
    }
    let bps = u16::from_le_bytes([b[0x0B], b[0x0C]]) as u32;
    let spc_raw = b[0x0D];
    let spc: u64 = if spc_raw > 0x80 { 1 << (256 - spc_raw as u32) } else { spc_raw as u64 };
    let cluster = bps as u64 * spc;
    let mft_lcn = u64::from_le_bytes(b[0x30..0x38].try_into().unwrap());
    let cpr = b[0x40] as i8;
    let record = if cpr < 0 { 1usize << (-cpr as u32) } else { cpr as usize * cluster as usize };
    if bps == 0 || cluster == 0 || !(256..=65536).contains(&record) {
        return Err("Neplatný boot sektor NTFS.".into());
    }
    Ok(Boot { bytes_per_sector: bps, cluster, mft_lcn, record })
}

/// Data runs → (lcn, clusters). Sparse runs come back with lcn = None.
pub fn parse_runs(mut r: &[u8]) -> Vec<(Option<u64>, u64)> {
    let mut out = Vec::new();
    let mut lcn: i64 = 0;
    while let Some(&h) = r.first() {
        if h == 0 {
            break;
        }
        let (ls, os) = ((h & 0x0F) as usize, (h >> 4) as usize);
        if ls == 0 || ls > 8 || os > 8 || r.len() < 1 + ls + os {
            break;
        }
        let mut len: u64 = 0;
        for i in 0..ls {
            len |= (r[1 + i] as u64) << (8 * i);
        }
        if os == 0 {
            out.push((None, len));
        } else {
            let mut off: i64 = 0;
            for i in 0..os {
                off |= (r[1 + ls + i] as i64) << (8 * i);
            }
            // sign-extend
            let shift = 64 - 8 * os as u32;
            off = (off << shift) >> shift;
            lcn += off;
            out.push((Some(lcn as u64), len));
        }
        r = &r[1 + ls + os..];
    }
    out
}

/// Applies the update-sequence fixup. Returns false for a damaged record.
pub fn fixup(rec: &mut [u8]) -> bool {
    if rec.len() < 48 || &rec[0..4] != b"FILE" {
        return false;
    }
    let off = u16::from_le_bytes([rec[4], rec[5]]) as usize;
    let count = u16::from_le_bytes([rec[6], rec[7]]) as usize;
    if count < 2 || off + count * 2 > rec.len() {
        return false;
    }
    let usn = [rec[off], rec[off + 1]];
    for i in 1..count {
        let end = i * 512;
        if end > rec.len() {
            break;
        }
        if rec[end - 2..end] != usn {
            return false;
        }
        rec[end - 2] = rec[off + i * 2];
        rec[end - 1] = rec[off + i * 2 + 1];
    }
    true
}

fn u16_at(b: &[u8], o: usize) -> u16 {
    b.get(o..o + 2).map(|x| u16::from_le_bytes([x[0], x[1]])).unwrap_or(0)
}
fn u32_at(b: &[u8], o: usize) -> u32 {
    b.get(o..o + 4).map(|x| u32::from_le_bytes(x.try_into().unwrap())).unwrap_or(0)
}
fn u64_at(b: &[u8], o: usize) -> u64 {
    b.get(o..o + 8).map(|x| u64::from_le_bytes(x.try_into().unwrap())).unwrap_or(0)
}

fn filetime_days(ft: u64) -> u32 {
    let secs = (ft / 10_000_000) as i64 - 11_644_473_600;
    if secs <= 0 { 0 } else { (secs / 86_400) as u32 }
}

/// What one record contributes. Extension records point to their base.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Parsed {
    pub base: Option<u32>,
    pub in_use: bool,
    pub dir: bool,
    /// (parent, name, namespace)
    pub name: Option<(u32, String, u8)>,
    pub size: Option<u64>,
    pub mtime: Option<u32>,
    pub cloud: bool,
    /// Data runs of the unnamed $DATA (only needed for $MFT itself).
    pub runs: Option<Vec<(Option<u64>, u64)>>,
}

const OFFLINE: u32 = 0x1000;
const RECALL_ON_OPEN: u32 = 0x40000;
const RECALL_ON_DATA_ACCESS: u32 = 0x40_0000;

/// Parses one FILE record (after fixup).
pub fn parse_record(rec: &[u8]) -> Parsed {
    let mut p = Parsed::default();
    let flags = u16_at(rec, 0x16);
    p.in_use = flags & 1 != 0;
    p.dir = flags & 2 != 0;
    let base = u64_at(rec, 0x20) & 0xFFFF_FFFF_FFFF;
    if base != 0 {
        p.base = Some(base as u32);
    }
    let used = (u32_at(rec, 0x18) as usize).min(rec.len());
    let mut a = u16_at(rec, 0x14) as usize;
    while a + 16 <= used {
        let ty = u32_at(rec, a);
        if ty == 0xFFFF_FFFF {
            break;
        }
        let len = u32_at(rec, a + 4) as usize;
        if len < 16 || a + len > used {
            break;
        }
        let attr = &rec[a..a + len];
        let non_resident = attr[8] != 0;
        let name_len = attr[9];
        let value = |attr: &[u8]| -> Option<Vec<u8>> {
            let vl = u32_at(attr, 0x10) as usize;
            let vo = u16_at(attr, 0x14) as usize;
            attr.get(vo..vo + vl).map(|v| v.to_vec())
        };
        match ty {
            0x10 if !non_resident => {
                if let Some(v) = value(attr) {
                    p.mtime = Some(filetime_days(u64_at(&v, 0x08)));
                    let fa = u32_at(&v, 0x20);
                    p.cloud = fa & (OFFLINE | RECALL_ON_OPEN | RECALL_ON_DATA_ACCESS) != 0;
                }
            }
            0x30 if !non_resident => {
                if let Some(v) = value(attr) {
                    let parent = (u64_at(&v, 0) & 0xFFFF_FFFF_FFFF) as u32;
                    let nl = v.get(0x40).copied().unwrap_or(0) as usize;
                    let ns = v.get(0x41).copied().unwrap_or(0);
                    if let Some(raw) = v.get(0x42..0x42 + nl * 2) {
                        let w: Vec<u16> = raw.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
                        let name = String::from_utf16_lossy(&w);
                        // Prefer the long Win32 name over the 8.3 DOS name.
                        let better = match &p.name {
                            None => true,
                            Some((_, _, old)) => *old == 2 && ns != 2,
                        };
                        if better {
                            p.name = Some((parent, name, ns));
                        }
                    }
                }
            }
            0x80 if name_len == 0 => {
                if non_resident {
                    if u64_at(attr, 0x10) == 0 {
                        p.size = Some(u64_at(attr, 0x30));
                        let ro = u16_at(attr, 0x20) as usize;
                        if let Some(r) = attr.get(ro..) {
                            p.runs = Some(parse_runs(r));
                        }
                    }
                } else {
                    p.size = Some(u32_at(attr, 0x10) as u64);
                }
            }
            _ => {}
        }
        a += len;
    }
    p
}

/// Merges parsed records into the volume. Extension records add their
/// $DATA size or names to the base record.
pub fn merge(records: Vec<(u32, Parsed)>) -> Volume {
    let max = records.iter().map(|(i, _)| *i).max().unwrap_or(0) as usize;
    let mut entries: Vec<Option<Entry>> = vec![None; max + 1];
    let mut extensions = Vec::new();
    for (i, p) in records {
        if !p.in_use {
            continue;
        }
        if let Some(b) = p.base {
            extensions.push((b, p));
            continue;
        }
        let Some((parent, name, _)) = p.name else { continue };
        entries[i as usize] = Some(Entry {
            parent,
            name,
            size: if p.dir || p.cloud { 0 } else { p.size.unwrap_or(0) },
            mtime: p.mtime.unwrap_or(0),
            dir: p.dir,
            cloud: p.cloud,
        });
    }
    for (b, p) in extensions {
        if let Some(Some(e)) = entries.get_mut(b as usize) {
            if let (Some(s), false) = (p.size, e.dir || e.cloud) {
                if e.size == 0 {
                    e.size = s;
                }
            }
        }
    }
    Volume { entries }
}

/// Reads the whole MFT of a volume.
pub fn read_volume<R: Read + Seek>(dev: &mut R, progress: &dyn Fn(u64)) -> Result<Volume, String> {
    let mut boot = vec![0u8; 512];
    dev.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
    dev.read_exact(&mut boot).map_err(|e| e.to_string())?;
    let b = parse_boot(&boot)?;

    // Record 0 is $MFT itself; its $DATA runs say where the rest is.
    let first_len = (b.record as u64).max(b.bytes_per_sector as u64);
    let mut r0 = vec![0u8; first_len as usize];
    dev.seek(SeekFrom::Start(b.mft_lcn * b.cluster)).map_err(|e| e.to_string())?;
    dev.read_exact(&mut r0).map_err(|e| e.to_string())?;
    let mut rec0 = r0[..b.record].to_vec();
    if !fixup(&mut rec0) {
        return Err("Neplatný záznam $MFT.".into());
    }
    let p0 = parse_record(&rec0);
    let runs = p0.runs.ok_or("V $MFT chýbajú data.")?;
    let mft_size = p0.size.unwrap_or(0);

    let mut parsed: Vec<(u32, Parsed)> = Vec::new();
    let mut index: u64 = 0;
    let total = mft_size / b.record as u64;
    const CHUNK: u64 = 8 << 20;
    'runs: for (lcn, clusters) in runs {
        let bytes = clusters * b.cluster;
        let Some(lcn) = lcn else {
            index += bytes / b.record as u64;
            continue;
        };
        let mut done = 0u64;
        while done < bytes {
            let n = (bytes - done).min(CHUNK);
            let mut buf = vec![0u8; n as usize];
            dev.seek(SeekFrom::Start(lcn * b.cluster + done)).map_err(|e| e.to_string())?;
            dev.read_exact(&mut buf).map_err(|e| e.to_string())?;
            use rayon::prelude::*;
            let start = index;
            let batch: Vec<(u32, Parsed)> = buf
                .par_chunks_mut(b.record)
                .enumerate()
                .filter_map(|(k, rec)| {
                    if rec.len() < b.record || !fixup(rec) {
                        return None;
                    }
                    Some(((start + k as u64) as u32, parse_record(rec)))
                })
                .collect();
            parsed.extend(batch);
            index += n / b.record as u64;
            done += n;
            progress(index.min(total));
            if index >= total {
                break 'runs;
            }
        }
    }
    Ok(merge(parsed))
}

// ---- compact file the elevated helper hands back

const MAGIC: &[u8; 6] = b"WIMFT1";

pub fn write_volume<W: Write>(v: &Volume, w: &mut W) -> std::io::Result<()> {
    w.write_all(MAGIC)?;
    w.write_all(&(v.entries.len() as u32).to_le_bytes())?;
    for (i, e) in v.entries.iter().enumerate() {
        let Some(e) = e else { continue };
        let name = e.name.as_bytes();
        w.write_all(&(i as u32).to_le_bytes())?;
        w.write_all(&e.parent.to_le_bytes())?;
        w.write_all(&e.size.to_le_bytes())?;
        w.write_all(&e.mtime.to_le_bytes())?;
        w.write_all(&[(e.dir as u8) | ((e.cloud as u8) << 1)])?;
        w.write_all(&(name.len().min(u16::MAX as usize) as u16).to_le_bytes())?;
        w.write_all(&name[..name.len().min(u16::MAX as usize)])?;
    }
    Ok(())
}

pub fn read_file(b: &[u8]) -> Result<Volume, String> {
    if b.len() < 10 || &b[..6] != MAGIC {
        return Err("Neplatný výsledok rýchleho skenu.".into());
    }
    let n = u32::from_le_bytes(b[6..10].try_into().unwrap()) as usize;
    let mut entries: Vec<Option<Entry>> = vec![None; n];
    let mut o = 10;
    while o + 23 <= b.len() {
        let i = u32_at(b, o) as usize;
        let parent = u32_at(b, o + 4);
        let size = u64_at(b, o + 8);
        let mtime = u32_at(b, o + 16);
        let fl = b[o + 20];
        let nl = u16_at(b, o + 21) as usize;
        o += 23;
        let Some(raw) = b.get(o..o + nl) else { break };
        o += nl;
        if i < n {
            entries[i] = Some(Entry {
                parent,
                name: String::from_utf8_lossy(raw).into_owned(),
                size,
                mtime,
                dir: fl & 1 != 0,
                cloud: fl & 2 != 0,
            });
        }
    }
    Ok(Volume { entries })
}

#[cfg(test)]
pub mod tests {
    use super::*;

    const REC: usize = 1024;
    const CL: usize = 4096;

    /// Builds a FILE record with the given attributes (fixup applied).
    pub fn record(flags: u16, base: u64, attrs: &[Vec<u8>]) -> Vec<u8> {
        let mut r = vec![0u8; REC];
        r[0..4].copy_from_slice(b"FILE");
        r[4..6].copy_from_slice(&0x30u16.to_le_bytes()); // USA offset
        r[6..8].copy_from_slice(&3u16.to_le_bytes()); // USN + 2 sectors
        r[0x14..0x16].copy_from_slice(&0x38u16.to_le_bytes());
        r[0x16..0x18].copy_from_slice(&flags.to_le_bytes());
        r[0x20..0x28].copy_from_slice(&base.to_le_bytes());
        let mut o = 0x38;
        for a in attrs {
            r[o..o + a.len()].copy_from_slice(a);
            o += a.len();
        }
        r[o..o + 4].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
        r[0x18..0x1C].copy_from_slice(&((o + 8) as u32).to_le_bytes());
        // Write the fixup: real bytes go to the USA, sector ends get the USN.
        let usn = [0x07, 0x00];
        r[0x30..0x32].copy_from_slice(&usn);
        for i in 1..3 {
            let end = i * 512;
            r[0x30 + i * 2] = r[end - 2];
            r[0x30 + i * 2 + 1] = r[end - 1];
            r[end - 2..end].copy_from_slice(&usn);
        }
        r
    }

    fn resident(ty: u32, name_len: u8, value: &[u8]) -> Vec<u8> {
        let len = (0x18 + value.len() + 7) & !7;
        let mut a = vec![0u8; len];
        a[0..4].copy_from_slice(&ty.to_le_bytes());
        a[4..8].copy_from_slice(&(len as u32).to_le_bytes());
        a[9] = name_len;
        a[0x10..0x14].copy_from_slice(&(value.len() as u32).to_le_bytes());
        a[0x14..0x16].copy_from_slice(&0x18u16.to_le_bytes());
        a[0x18..0x18 + value.len()].copy_from_slice(value);
        a
    }

    pub fn file_name(parent: u64, name: &str, ns: u8) -> Vec<u8> {
        let w: Vec<u16> = name.encode_utf16().collect();
        let mut v = vec![0u8; 0x42 + w.len() * 2];
        v[0..8].copy_from_slice(&(parent | (1 << 48)).to_le_bytes());
        v[0x40] = w.len() as u8;
        v[0x41] = ns;
        for (i, c) in w.iter().enumerate() {
            v[0x42 + i * 2..0x44 + i * 2].copy_from_slice(&c.to_le_bytes());
        }
        resident(0x30, 0, &v)
    }

    pub fn std_info(days_since_1970: u64, attrs: u32) -> Vec<u8> {
        let mut v = vec![0u8; 0x48];
        let ft = (days_since_1970 * 86_400 + 11_644_473_600) * 10_000_000;
        v[0x08..0x10].copy_from_slice(&ft.to_le_bytes());
        v[0x20..0x24].copy_from_slice(&attrs.to_le_bytes());
        resident(0x10, 0, &v)
    }

    pub fn data_resident(size: usize) -> Vec<u8> {
        resident(0x80, 0, &vec![0u8; size])
    }

    pub fn data_nonresident(real: u64, runs: &[u8]) -> Vec<u8> {
        let len = (0x40 + runs.len() + 1 + 7) & !7;
        let mut a = vec![0u8; len];
        a[0..4].copy_from_slice(&0x80u32.to_le_bytes());
        a[4..8].copy_from_slice(&(len as u32).to_le_bytes());
        a[8] = 1;
        a[0x20..0x22].copy_from_slice(&0x40u16.to_le_bytes());
        a[0x28..0x30].copy_from_slice(&real.to_le_bytes());
        a[0x30..0x38].copy_from_slice(&real.to_le_bytes());
        a[0x40..0x40 + runs.len()].copy_from_slice(runs);
        a
    }

    /// A tiny NTFS image: boot sector, MFT at cluster 4 with 24 records in
    /// two fragments (records 0–15 at cluster 4, 16–23 at cluster 20).
    pub fn image(records: Vec<(usize, Vec<u8>)>) -> Vec<u8> {
        let mut img = vec![0u8; CL * 40];
        let b = &mut img[0..512];
        b[3..11].copy_from_slice(b"NTFS    ");
        b[0x0B..0x0D].copy_from_slice(&512u16.to_le_bytes());
        b[0x0D] = 8; // 4 KB clusters
        b[0x30..0x38].copy_from_slice(&4u64.to_le_bytes());
        b[0x40] = 0xF6; // -10 → 1024-byte records
        // $MFT runs: 4 clusters at LCN 4, then 2 clusters at LCN 20 (+16).
        let runs = [0x11, 0x04, 0x04, 0x11, 0x02, 0x10, 0x00];
        let mut all = records;
        all.push((0, record(1, 0, &[std_info(19000, 0), file_name(5, "$MFT", 3), data_nonresident(24 * REC as u64, &runs)])));
        for (i, r) in all {
            let off = if i < 16 { 4 * CL + i * REC } else { 20 * CL + (i - 16) * REC };
            img[off..off + REC].copy_from_slice(&r);
        }
        img
    }

    #[test]
    fn decodes_boot_and_runs() {
        let img = image(vec![]);
        let b = parse_boot(&img[..512]).unwrap();
        assert_eq!((b.cluster, b.mft_lcn, b.record), (4096, 4, 1024));
        assert_eq!(parse_runs(&[0x11, 0x04, 0x04, 0x11, 0x02, 0x10, 0x00]), vec![(Some(4), 4), (Some(20), 2)]);
        // Negative offset and a sparse run.
        assert_eq!(parse_runs(&[0x11, 0x01, 0x30, 0x11, 0x01, 0xF0, 0x01, 0x05, 0x00]), vec![(Some(0x30), 1), (Some(0x20), 1), (None, 5)]);
        assert!(parse_boot(&[0u8; 512]).is_err());
    }

    #[test]
    fn rejects_torn_records() {
        let mut r = record(1, 0, &[file_name(5, "a", 1)]);
        r[1022] ^= 0xFF; // sector end no longer matches the USN
        assert!(!fixup(&mut r));
    }

    #[test]
    fn reads_a_volume() {
        let recs = vec![
            (5, record(3, 0, &[std_info(19000, 0), file_name(5, ".", 3)])),
            (16, record(3, 0, &[std_info(19500, 0), file_name(5, "Games", 1)])),
            // Long Win32 name after the DOS name wins.
            (17, record(1, 0, &[std_info(19600, 0), file_name(16, "BIGGAM~1.PAK", 2), file_name(16, "big game.pak", 1), data_nonresident(5 << 30, &[0x11, 0x01, 0x08, 0x00])])),
            (18, record(1, 0, &[std_info(19700, 0), file_name(5, "note.txt", 3), data_resident(300)])),
            // OneDrive placeholder: size doesn't count.
            (19, record(1, 0, &[std_info(19700, 0x40_0000), file_name(5, "cloud.mp4", 3), data_nonresident(9 << 30, &[0x11, 0x01, 0x09, 0x00])])),
            // Deleted record (not in use) is ignored.
            (20, record(0, 0, &[file_name(5, "gone.bin", 3), data_resident(10)])),
            // Base record whose $DATA lives in an extension record (21 → 22).
            (21, record(1, 0, &[std_info(19700, 0), file_name(16, "fragmented.bin", 1)])),
            (22, record(1, 21, &[data_nonresident(7 << 20, &[0x11, 0x01, 0x0A, 0x00])])),
        ];
        let img = image(recs);
        let mut cur = std::io::Cursor::new(img);
        let v = read_volume(&mut cur, &|_| {}).unwrap();
        let e = |i: usize| v.entries[i].as_ref().unwrap();
        assert!(e(5).dir);
        assert_eq!(e(16).name, "Games");
        assert_eq!(e(17).name, "big game.pak");
        assert_eq!(e(17).parent, 16);
        assert_eq!(e(17).size, 5 << 30);
        assert_eq!(e(17).mtime, 19600);
        assert_eq!(e(18).size, 300);
        assert!(e(19).cloud && e(19).size == 0);
        assert!(v.entries[20].is_none());
        assert!(v.entries[22].is_none());
        assert_eq!(e(21).size, 7 << 20);
        assert_eq!(e(0).name, "$MFT");

        // Round trip through the helper's file.
        let mut buf = Vec::new();
        write_volume(&v, &mut buf).unwrap();
        let back = read_file(&buf).unwrap();
        assert_eq!(back.entries, v.entries);
    }
}
