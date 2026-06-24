use std::{collections::HashMap, fs::File, io::Read};

use camino::{Utf8Path, Utf8PathBuf};
use rawzip::{
    time::ZipDateTimeKind, CompressionMethod, FileReader, ZipArchive, ZipArchiveEntryWayfinder,
};

use crate::{hash, manager::ModConfig, ModError};

pub trait VirtualFS: Sync + Send {
    fn root(&self) -> &Utf8Path;
    fn entries(&self) -> &HashMap<u32, Utf8PathBuf>;
    fn load(&self, hash: u32) -> Result<Vec<u8>, ModError>;
    fn last_modified(&self, hash: u32) -> Result<u64, ModError>;
    fn is_writeable(&self) -> bool;
    fn write(&self, rel: &Utf8Path, bytes: &[u8]) -> Result<(), ModError>;
}

impl dyn VirtualFS {
    pub fn canonical_path(&self, hash: u32) -> Option<&Utf8Path> {
        self.entries().get(&hash).map(Utf8PathBuf::as_path)
    }

    pub(crate) fn config(&self) -> Result<ModConfig, ModError> {
        let bytes = self.load(hash("config.yaml"))?;
        serde_yaml::from_slice(&bytes).map_err(ModError::Config)
    }
}

fn collect_dir_entries(root: &Utf8Path) -> HashMap<u32, Utf8PathBuf> {
    let mut entries = HashMap::new();
    let mut stack: Vec<Utf8PathBuf> = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let Ok(read) = std::fs::read_dir(&dir) else { continue };
        for entry in read.flatten() {
            let Ok(file_type) = entry.file_type() else { continue };
            let Ok(entry_path) = Utf8PathBuf::from_path_buf(entry.path()) else { continue };

            if file_type.is_dir() {
                stack.push(entry_path);
            } else if file_type.is_file() && entry_path.extension().is_some() {
                let Ok(rel) = entry_path.strip_prefix(root) else { continue };
                let rel = rel.to_path_buf();
                entries.insert(hash(&rel), rel);
            }
        }
    }

    entries
}

pub struct ModDir {
    root: Utf8PathBuf,
    entries: HashMap<u32, Utf8PathBuf>,
}

impl ModDir {
    pub fn new(root: impl AsRef<Utf8Path>) -> Self {
        let root = root.as_ref().to_path_buf();
        let entries = collect_dir_entries(&root);
        Self { root, entries }
    }
}

impl VirtualFS for ModDir {
    fn root(&self) -> &Utf8Path {
        &self.root
    }

    fn entries(&self) -> &HashMap<u32, Utf8PathBuf> {
        &self.entries
    }

    fn load(&self, hash: u32) -> Result<Vec<u8>, ModError> {
        let canonical = self.entries.get(&hash).ok_or(ModError::NotFound)?;
        std::fs::read(self.root.join(canonical)).map_err(ModError::Io)
    }

    fn last_modified(&self, hash: u32) -> Result<u64, ModError> {
        let canonical = self.entries.get(&hash).ok_or(ModError::NotFound)?;
        let full_path = self.root.join(canonical);
        let mut timestamp = nnsdk::fs::FileTimeStamp::new();
        let filepath = std::ffi::CString::new(full_path.to_string()).unwrap();
        unsafe { nnsdk::fs::GetFileTimeStampForDebug(&mut timestamp, filepath.as_c_str().to_bytes_with_nul().as_ptr()) };

        Ok(timestamp.modify.time)
    }

    fn is_writeable(&self) -> bool {
        true
    }

    fn write(&self, rel: &Utf8Path, bytes: &[u8]) -> Result<(), ModError> {
        let full = self.root.join(rel);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).map_err(ModError::Io)?;
        }
        std::fs::write(&full, bytes).map_err(ModError::Io)
    }
}

const ZIP_SCRATCH_SIZE: usize = 64 * 1024;

struct ZipEntryMeta {
    wayfinder: ZipArchiveEntryWayfinder,
    compression: CompressionMethod,
    mtime: u64,
}

fn pack_mtime(kind: ZipDateTimeKind) -> u64 {
    let (y, mo, d, h, mi, s) = match kind {
        ZipDateTimeKind::Utc(dt) => (dt.year(), dt.month(), dt.day(), dt.hour(), dt.minute(), dt.second()),
        ZipDateTimeKind::Local(dt) => (dt.year(), dt.month(), dt.day(), dt.hour(), dt.minute(), dt.second()),
    };
    ((y as u64) << 40) | ((mo as u64) << 32) | ((d as u64) << 24) | ((h as u64) << 16) | ((mi as u64) << 8) | (s as u64)
}

pub struct ZippedMod {
    root: Utf8PathBuf,
    entries: HashMap<u32, Utf8PathBuf>,
    meta: HashMap<u32, ZipEntryMeta>,
    archive: ZipArchive<FileReader>,
}

impl ZippedMod {
    pub fn new(root: impl AsRef<Utf8Path>) -> Self {
        let root = root.as_ref().to_path_buf();
        let file = File::open(&root).unwrap_or_else(|e| panic!("Could not open zipped mod '{}': {}", root, e));

        let mut scratch = vec![0u8; ZIP_SCRATCH_SIZE];
        let archive = ZipArchive::from_file(file, &mut scratch)
            .unwrap_or_else(|e| panic!("Could not parse zipped mod '{}': {:?}", root, e));

        let mut entries: HashMap<u32, Utf8PathBuf> = HashMap::new();
        let mut meta: HashMap<u32, ZipEntryMeta> = HashMap::new();

        let mut iter = archive.entries(&mut scratch);
        loop {
            let entry = match iter.next_entry() {
                Ok(Some(entry)) => entry,
                Ok(None) => break,
                Err(e) => {
                    println!("Warning: error iterating zip entries in '{}': {:?}", root, e);
                    break;
                },
            };

            if entry.is_dir() {
                continue;
            }

            let Ok(normalized) = entry.file_path().try_normalize() else { continue };
            let path = Utf8PathBuf::from(normalized.as_ref());

            if path.extension().is_none() || path.starts_with("__MACOSX") {
                continue;
            }

            let h = hash(&path);
            meta.insert(
                h,
                ZipEntryMeta {
                    wayfinder: entry.wayfinder(),
                    compression: entry.compression_method(),
                    mtime: pack_mtime(entry.last_modified()),
                },
            );
            entries.insert(h, path);
        }

        Self { root, entries, meta, archive }
    }
}

impl VirtualFS for ZippedMod {
    fn root(&self) -> &Utf8Path {
        &self.root
    }

    fn entries(&self) -> &HashMap<u32, Utf8PathBuf> {
        &self.entries
    }

    fn load(&self, hash: u32) -> Result<Vec<u8>, ModError> {
        let meta = self.meta.get(&hash).ok_or(ModError::NotFound)?;
        let local = self
            .archive
            .get_entry(meta.wayfinder)
            .map_err(|e| ModError::Io(std::io::Error::other(format!("rawzip get_entry: {:?}", e))))?;

        let mut out = Vec::with_capacity(meta.wayfinder.uncompressed_size_hint() as usize);

        match meta.compression {
            CompressionMethod::Store => {
                let raw = local.reader();
                let mut reader = local.verifying_reader(raw);
                reader.read_to_end(&mut out).map_err(ModError::Io)?;
            },
            CompressionMethod::Deflate => {
                let decoder = flate2::read::DeflateDecoder::new(local.reader());
                let mut reader = local.verifying_reader(decoder);
                reader.read_to_end(&mut out).map_err(ModError::Io)?;
            },
            other => {
                return Err(ModError::Io(std::io::Error::other(format!(
                    "unsupported compression method in zipped mod: {:?}",
                    other
                ))));
            },
        }

        Ok(out)
    }

    fn last_modified(&self, hash: u32) -> Result<u64, ModError> {
        Ok(self.meta.get(&hash).ok_or(ModError::NotFound)?.mtime)
    }

    fn is_writeable(&self) -> bool {
        false
    }

    fn write(&self, _rel: &Utf8Path, _bytes: &[u8]) -> Result<(), ModError> {
        Err(ModError::ReadOnly)
    }
}
