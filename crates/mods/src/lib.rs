#![feature(test)]

use std::{collections::{hash_map::DefaultHasher, HashMap}, hash::Hasher, sync::Arc};


use camino::{Utf8Path, Utf8PathBuf};
use vfs::{ModDir, ZippedMod, VirtualFS};
use walkdir::WalkDir;

use thiserror::Error;

mod bucket_map;
mod builder;
mod interner;
pub mod manager;
mod vfs;

pub fn hash(path: impl AsRef<Utf8Path>) -> u32 {
    let mut hash = DefaultHasher::new();
    hash.write(path.as_ref().as_str().to_lowercase().as_bytes());
    hash.finish() as u32
}

#[derive(Debug, Error)]
pub enum ModError {
    #[error("a IO error happened")]
    IoError(#[from] std::io::Error),
    #[error("a u32 for a file could not be found: {:#08x}", .0)]
    Missingu32(u32),
    #[error("a u32 for a directory could not be found: {:#08x}", .0)]
    MissingDirectoryu32(u32),
    #[error("the cached file size for hash {:#08x} is different from the physical file: cache({1}) != storage({2})", .0)]
    FilesizeMismatch(u32, usize, usize),
    #[error("the requested file is not found in the Virtual FS")]
    MissingFile,
    #[error("a mod with this hash already exists")]
    AlreadyExists,
    #[error("the configuration file could not be read")]

    ConfigError(#[from] serde_yaml::Error),
}

pub fn discover_in_mods<P: AsRef<Utf8Path>>(root: P) -> Vec<(Utf8PathBuf, Utf8PathBuf)> {
    let root = root.as_ref();

    WalkDir::new(root)
        .min_depth(1)
        .into_iter()
        .flatten()
        .flat_map(|entry| {
            // Ignore the directories, only care about the files that have an extension
            if entry.file_type().is_file() && entry.path().extension().is_some() {
                let relative_path = Utf8PathBuf::from_path_buf(entry.path().strip_prefix(root).unwrap().into()).unwrap();
                Some((relative_path, Utf8PathBuf::from_path_buf(entry.path().into()).unwrap()))
            } else {
                None
            }
        })
        .collect()
}

pub fn discover_mods<P: AsRef<Utf8Path>>(root: P) -> HashMap<Utf8PathBuf, Vec<Utf8PathBuf>> {
    let root = root.as_ref();

    let mut vecs: Vec<(Utf8PathBuf, Utf8PathBuf)> = WalkDir::new(root)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_entry(|entry| entry.file_type().is_dir() && !Utf8Path::from_path(entry.path()).unwrap().file_name().unwrap().starts_with('.'))
        .flatten()
        .flat_map(|entry| discover_in_mods(Utf8Path::from_path(entry.path()).unwrap()))
        .collect();

    let mut hashmap: HashMap<Utf8PathBuf, Vec<Utf8PathBuf>> = HashMap::new();

    vecs.sort();

    vecs.into_iter().for_each(|(relative, absolute)| {
        // println!("Relative: {}, Absolute: {}", relative, absolute);
        match hashmap.get_mut(&relative) {
            Some(vec) => vec.push(absolute),
            None => {
                hashmap.insert(relative, vec![absolute]);
            },
        }
    });

    hashmap
}

pub fn discover_mods_manager<P: AsRef<Utf8Path>>(root: P) -> impl Iterator<Item = Arc<dyn VirtualFS>> {
    WalkDir::new(root.as_ref())
        .sort_by(|a, b| a.path().cmp(b.path()))
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_entry(|entry| !entry.file_name().to_str().unwrap().starts_with('.') && entry.file_type().is_dir() ||  !entry.file_name().to_str().unwrap().starts_with('.') && Utf8Path::from_path(entry.path()).unwrap().extension() == Some("zip"))
        .flatten()
        .flat_map(|entry| -> Option<Arc<dyn VirtualFS>> {
            let path = Utf8Path::from_path(entry.path()).unwrap();

            if path.is_dir() {
                Some(Arc::new(ModDir::new(path)))
            } else if path.extension() == Some("zip") {
                Some(Arc::new(ZippedMod::new(path)))
            } else {
                None
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
    fn get_locations() {
        let manager = Manager::get();
        let out = manager.get_locations().collect::<Vec<_>>();

        dbg!(&out);
        println!("Out len: {:x}", out.len());

        dbg!(&manager.get_full_path("patches/xml/AssetTable.xml").unwrap());
    }

    #[test]
    fn get_directory() {
        let manager = Manager::get();

        let dir = manager.get_directory("patches/xml").unwrap();
        let parent_dir = manager.get_parent_directory(dir).unwrap();

        assert_eq!(dir.parent, parent_dir.path.hash);
        assert_eq!(dir.child_dir_hashes.len(), 0);
        assert_eq!(dir.file_hashes.len(), 3);
    }

    #[test]
    fn get_directory_parent() {
        let manager = Manager::get();

        let dir = manager.get_directory("patches/xml").unwrap();
        let parent_dir = manager.get_parent_directory(dir).unwrap();

        let expected = manager.get_directory("patches").unwrap();

        assert_eq!(parent_dir, expected)
    }

    #[test]
    fn get_files_in_directory() {
        let manager = Manager::get();

        let dir = manager.get_directory("patches/xml").unwrap();

        let dir_files = manager.get_files_in_directory(dir).unwrap();

        let expected = vec![
            Utf8PathBuf::from("patches/xml/Shop.xml"),
            Utf8PathBuf::from("patches/xml/Item.xml"),
            Utf8PathBuf::from("patches/xml/AssetTable.xml")
        ];

        assert_eq!(dir_files, expected)
    }

    #[test]
    fn get_files_in_directory_and_subdir() {
        let manager = Manager::get();

        let dir = manager.get_directory("patches/msbt/message/us").unwrap();

        let dir_files = manager.get_files_in_directory_and_subdir(dir).unwrap();

        let expected = vec![
            Utf8PathBuf::from("patches/msbt/message/us/usfr/accessories.msbt"),
            Utf8PathBuf::from("patches/msbt/message/us/uses/accessories.msbt"),
            Utf8PathBuf::from("patches/msbt/message/us/usen/accessories.msbt")
        ];

        assert_eq!(dir_files, expected)
    }

    // Best time: 174ns
    #[bench]
    fn bench_get_full_path_original(b: &mut Bencher) {
        let manager = Manager::get();

        b.iter(|| manager.get_full_path_original("patches/xml/AssetTable.xml").unwrap());
    }

    // Best time: 152ns
    #[bench]
    fn bench_get_full_path(b: &mut Bencher) {
        let manager = Manager::get();

        b.iter(|| manager.get_full_path("patches/xml/AssetTable.xml").unwrap());
    }

    // Best time: 60ns
    #[bench]
    fn bench_get_directory(b: &mut Bencher) {
        let manager = Manager::get();

        b.iter(|| manager.get_directory("patches/xml").unwrap());
    }

    // Best time: 86ns
    #[bench]
    fn bench_get_files_in_empty_directory(b: &mut Bencher) {
        let manager = Manager::get();

        let dir = manager.get_directory("patches/msbt").unwrap();


        b.iter(|| manager.get_files_in_directory(dir).unwrap());
    }

    // Best time: 157ns
    #[bench]
    fn bench_get_file_in_directory(b: &mut Bencher) {
        let manager = Manager::get();

        let dir = manager.get_directory("patches/msbt/message/us/usen").unwrap();


        b.iter(|| manager.get_files_in_directory(dir).unwrap());
    }

    // Best time: 392ns
    #[bench]
    fn bench_get_files_in_directory(b: &mut Bencher) {
        let manager = Manager::get();

        let dir = manager.get_directory("patches/xml").unwrap();


        b.iter(|| manager.get_files_in_directory(dir).unwrap());
    }

    // Best time: 2259ns
    #[bench]
    fn bench_get_files_in_directory_and_subdirs(b: &mut Bencher) {
        let manager = Manager::get();

        let dir = manager.get_directory("patches/msbt").unwrap();


        b.iter(|| manager.get_files_in_directory_and_subdir(dir).unwrap());
    }
}