use camino::Utf8Path;

use crate::{hash, manager::DirectoryInfo};

#[derive(Debug, Default)]
pub struct FilesystemBuilder {
    dir_infos: Vec<DirectoryInfo>,
}

impl FilesystemBuilder {
    pub fn new() -> Self {
        let mut this = Self::default();
        this.dir_infos.push(DirectoryInfo::new(hash("")));
        this
    }

    pub fn finish(mut self) -> Vec<DirectoryInfo> {
        self.dir_infos.sort_by_key(|d| d.path);
        self.dir_infos
    }

    pub fn add_file(&mut self, path: &Utf8Path, path_hash: u32) {
        let parent = path.parent().unwrap_or_else(|| Utf8Path::new(""));
        let parent_hash = hash(parent);

        if self.dir(parent_hash).is_none() {
            self.create_directory_hierarchy(parent);
        }

        self.dir_mut(parent_hash)
            .expect("parent directory must exist after creation")
            .file_hashes
            .push(path_hash);
    }

    fn dir(&self, path_hash: u32) -> Option<&DirectoryInfo> {
        self.dir_infos.iter().find(|d| d.path == path_hash)
    }

    fn dir_mut(&mut self, path_hash: u32) -> Option<&mut DirectoryInfo> {
        self.dir_infos.iter_mut().find(|d| d.path == path_hash)
    }

    fn add_directory(&mut self, path: &Utf8Path) {
        let path_hash = hash(path);
        let parent_hash = hash(path.parent().unwrap_or_else(|| Utf8Path::new("")));

        self.dir_mut(parent_hash)
            .unwrap_or_else(|| panic!("missing parent directory when adding {}", path))
            .child_dir_hashes
            .push(path_hash);

        self.dir_infos.push(DirectoryInfo::new(path_hash));
    }

    fn create_directory_hierarchy(&mut self, path: &Utf8Path) {
        let parent = path.parent().expect("non-root path should have a parent");
        if self.dir(hash(parent)).is_none() {
            self.create_directory_hierarchy(parent);
        }
        self.add_directory(path);
    }
}
