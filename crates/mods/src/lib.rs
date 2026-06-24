#![cfg_attr(test, feature(test))]

use std::{
    collections::hash_map::DefaultHasher,
    hash::Hasher,
    sync::Arc,
};

use camino::{Utf8Path, Utf8PathBuf};
use thiserror::Error;
use manager::Manager;
use vfs::{ModDir, VirtualFS, ZippedMod};

pub use manager::Mod;

mod builder;
pub mod manager;
mod vfs;

pub fn read(path: impl AsRef<Utf8Path>) -> Result<Vec<u8>, ModError> {
    Manager::get().read(path)
}

pub fn read_layers(path: impl AsRef<Utf8Path>) -> Result<Vec<Vec<u8>>, ModError> {
    Manager::get().read_layers(path)
}

pub fn contains(path: impl AsRef<Utf8Path>) -> bool {
    Manager::get().contains(path)
}

pub fn contains_directory(path: impl AsRef<Utf8Path>) -> bool {
    Manager::get().contains_directory(path)
}

pub fn list(dir: impl AsRef<Utf8Path>) -> impl Iterator<Item = &'static Utf8Path> {
    Manager::get().list(dir)
}

pub fn list_recursive(dir: impl AsRef<Utf8Path>) -> impl Iterator<Item = &'static Utf8Path> {
    Manager::get().list_recursive(dir)
}

pub fn disk_path(path: impl AsRef<Utf8Path>) -> Option<Utf8PathBuf> {
    Manager::get().disk_path(path)
}

pub fn last_modified(path: impl AsRef<Utf8Path>) -> Result<u64, ModError> {
    Manager::get().last_modified(path)
}

pub fn mods() -> impl Iterator<Item = &'static Mod> {
    Manager::get().mods()
}

pub fn hash(path: impl AsRef<Utf8Path>) -> u32 {
    let mut hash = DefaultHasher::new();
    hash.write(path.as_ref().as_str().to_lowercase().as_bytes());
    hash.finish() as u32
}

#[derive(Debug, Error)]
pub enum ModError {
    #[error("the requested entry is not present in any mod")]
    NotFound,
    #[error("this mod is read-only and cannot be written to")]
    ReadOnly,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("failed to parse mod configuration: {0}")]
    Config(#[from] serde_yaml::Error),
}

pub fn discover_mods_manager<P: AsRef<Utf8Path>>(root: P) -> impl Iterator<Item = Arc<dyn VirtualFS>> {
    let mut entries: Vec<(Utf8PathBuf, bool)> = std::fs::read_dir(root.as_ref())
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let path = Utf8PathBuf::from_path_buf(entry.path()).ok()?;
            let name = path.file_name()?;
            if name.starts_with('.') {
                return None;
            }
            let file_type = entry.file_type().ok()?;
            if file_type.is_dir() {
                Some((path, true))
            } else if file_type.is_file() && path.extension() == Some("zip") {
                Some((path, false))
            } else {
                None
            }
        })
        .collect();

    entries.sort_by(|a, b| a.0.cmp(&b.0));

    entries.into_iter().map(|(path, is_dir)| -> Arc<dyn VirtualFS> {
        if is_dir {
            Arc::new(ModDir::new(&path))
        } else {
            Arc::new(ZippedMod::new(&path))
        }
    })
}

#[cfg(test)]
mod tests {
    extern crate test;

    use camino::Utf8PathBuf;
    use test::Bencher;

    use crate::manager::Manager;

    #[test]
    fn list_xml_directory() {
        let manager = Manager::get();
        let files: Vec<Utf8PathBuf> = manager.list("patches/xml").map(|p| p.to_path_buf()).collect();

        let expected = vec![
            Utf8PathBuf::from("patches/xml/Shop.xml"),
            Utf8PathBuf::from("patches/xml/Item.xml"),
            Utf8PathBuf::from("patches/xml/AssetTable.xml"),
        ];

        assert_eq!(files, expected);
    }

    #[test]
    fn list_recursive_msbt() {
        let manager = Manager::get();
        let files: Vec<Utf8PathBuf> = manager.list_recursive("patches/msbt/message/us").map(|p| p.to_path_buf()).collect();

        let expected = vec![
            Utf8PathBuf::from("patches/msbt/message/us/usfr/accessories.msbt"),
            Utf8PathBuf::from("patches/msbt/message/us/uses/accessories.msbt"),
            Utf8PathBuf::from("patches/msbt/message/us/usen/accessories.msbt"),
        ];

        assert_eq!(files, expected);
    }

    #[test]
    fn case_insensitive_lookup() {
        let manager = Manager::get();
        assert_eq!(manager.contains("patches/xml/AssetTable.xml"), manager.contains("patches/xml/assettable.xml"));
    }

    #[bench]
    fn bench_read(b: &mut Bencher) {
        let manager = Manager::get();
        b.iter(|| manager.read("patches/xml/AssetTable.xml").unwrap());
    }

    #[bench]
    fn bench_contains(b: &mut Bencher) {
        let manager = Manager::get();
        b.iter(|| manager.contains("patches/xml/AssetTable.xml"));
    }

    #[bench]
    fn bench_list(b: &mut Bencher) {
        let manager = Manager::get();
        b.iter(|| manager.list("patches/xml").count());
    }

    #[bench]
    fn bench_list_recursive(b: &mut Bencher) {
        let manager = Manager::get();
        b.iter(|| manager.list_recursive("patches/msbt").count());
    }
}
