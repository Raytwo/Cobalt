// Inspired by the Addressable system. The parent class for everything else

use std::sync::{Arc, LazyLock};

use camino::{Utf8PathBuf, Utf8Path};
use dependency_graph::{DependencyGraph, Node, Step};
use multimap::MultiMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{discover_mods_manager, vfs::VirtualFS, ModError, interner::HashedPathInterner, builder::FilesystemBuilder, hash};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct ModConfig {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) author: String,
    #[serde(default)]
    pub(crate) dependencies: Vec<String>,
    pub(crate) repository: Option<String>
}

pub struct ModPair {
    config: ModConfig,
    vfs: Arc<dyn VirtualFS>
}

impl Node for ModPair {
    type DependencyType = String;

    fn dependencies(&self) -> &[Self::DependencyType] {
        &self.config.dependencies
    }

    fn matches(&self, dependency: &Self::DependencyType) -> bool {
        self.config.id == *dependency
    }
}

// That'd mean we have to make sure every file and child directory are following each others
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct DirectoryInfo {
    pub path: HashToIndex,
    // Hash
    pub parent: u32,
    pub file_hashes: Vec<u32>,
    pub child_dir_hashes: Vec<u32>,
}

impl DirectoryInfo {
    pub fn new(path: HashToIndex, parent: u32) -> Self {
        Self {
            path,
            parent,
            .. Default::default()
        }
    }

    pub fn new_from_resourcepath(resource_path: &ResourcePath) -> Self {
        Self {
            path: resource_path.path,
            parent: resource_path.parent.hash,
            .. Default::default()
        }
    }
}

#[derive(Debug, Error)]
pub enum LookupError {
    #[error("test")]
    Missing
}

// Logically speaking, you'd first search for the index to the _entries table and then use the SearchEntry to find what you need in the Manager
#[derive(Default)]
pub struct SearchSection {
    folders: Vec<HashToIndex>,
    folder_entries: Vec<SearchEntry>,
    files: Vec<HashToIndex>,
    file_entries: Vec<SearchEntry>,
}

impl SearchSection {
    fn get_folder_by_hash(&self, hash: u32) -> Result<&HashToIndex, LookupError> {
        if let Ok(hashtoindex) = self.folders.binary_search_by_key(&hash, |h2i| h2i.hash) {
            Ok(&self.folders[hashtoindex])
        } else {
            Err(LookupError::Missing)
        }
    }
    fn get_folder_entry_by_hash(&self, hash: u32) -> Result<&SearchEntry, LookupError> {
        self.get_folder_by_hash(hash).map(|h2i| &self.folder_entries[h2i.index as usize])
    }
}

// Pair a hash with the relevant index for its path
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct HashToIndex {
    pub hash: u32,
    pub index: u32,
}

// Assuming the table containing these is sorted by hash the hash of the path, binary search on the hash is possible
pub struct SearchEntry {
    path: HashToIndex,
    parent: HashToIndex,
    filename: HashToIndex,
    ext: HashToIndex,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourcePath {
    pub path: HashToIndex,
    pub parent: HashToIndex,
    pub filename: HashToIndex,
    pub ext: HashToIndex,
}

impl ResourcePath {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_from_path(path: impl AsRef<Utf8Path>) -> Self {
        let path = path.as_ref();

        let mut new_path = Self::new();

        new_path.path.hash = hash(path);
        // Directories don't have extensions
        new_path.ext.hash = path.extension().map(hash).unwrap_or(0xFFFF_FFFF);
        new_path.filename.hash = path.file_name().map(hash).unwrap_or(0);
        new_path.parent.hash = path.parent().map(hash).unwrap_or(0);

        new_path
    }
}

pub struct Manager {
    vfs: Vec<Arc<dyn VirtualFS>>,
    interner: HashedPathInterner<16>,
    lookup: MultiMap<u32, usize>,
    paths: Vec<ResourcePath>,
    dir_infos: Vec<DirectoryInfo>,
}

// TODO: Something to tie the keys to the resource(s) that were found.
// Do we store the keys with the full paths, or to a matching locator?



static MANAGER: LazyLock<Manager> = LazyLock::new(|| {
    // We'll want to get a unique mod entity (the directory with the files, the zip, ...) on each of the possible storages
    // Locators should be the ones exploring a storage (VirtualFS?) and it's directory
    // Providers are the ones who take a key and fetch it from the storage source, and perform extra operations on them (MSBT patching?) before returning the Vec<u8>/whatever format?
    // Should providers be stored in the Manager or be a separate class like Unity? Like MsbtProvider giving you a MessageMap and not a Vec<u8>?
    // Who stores the key to full path map? The Manager? Each Locator stores the paths it owns? Path storage needs to be optimized through a Interner, so a common place would be better.
    // What if we want to read a directory on one of the storages? Get files from their extension?
    // How do we search for something and fast? Something that can be sped up by a binary search would be nice, but that'd require sorting and reducing the possible range. Buckets might be useful here.
    // If the locators keep track of the files they found, we need a way to know which locator contains what file so we don't walk through each of them every time we need a file.
    // It is necessary to know ASAP if we own a file or not so hooks like IRawBundle don't slow down the game
    // So the manager should keep a list of keys with the ID of the appropriate locator(s) in another list
    // To deal with lowercase/uppercase issues, it might be better to keep a hash that's always lowercase, but be able to look for entries by a case-sensitive key too.
    // Or actually, always look by lowercase hash, so we ignore casing when it's about finding something
    // Looking into the HashToIndex from data.arc might be a solution to tie the hashes to the key/locator without taking up a ton more space.
    // One table for hashes to Locator ID, one table for keys to Locator ID, with a common Vec for both.
    // For now, let's just worry about SD and ZIPs files


    // First, we want to walk in sd:/engage/mods and get back Locators based on whether it's a directory or a ZIP.
    // Ultimately this isn't the job of the Locator, but for now...
    let mods = discover_mods_manager("sd:/engage/mods").collect::<Vec<_>>();

    let configs: Vec<ModPair> = mods.iter().map(|vfs| {
        let config = match vfs.get_config() {
            Ok(conf) => conf,
            Err(ModError::ConfigError(_))  => {
                panic!("Mod '{}' ran into a configuration error. Make sure the file is following the YAML specifications.", vfs.get_root().file_name().unwrap());
            },
            Err(_) => ModConfig::default(),
        };

        ModPair {
            config,
            vfs: vfs.clone()
        }
    }).collect();

    let mut dependants = Vec::new();
    let mut standalone = Vec::new();
    
    // Split between mods with and without configutation
    configs.into_iter()
        .for_each(|pair| {
            if pair.config.id.is_empty() {
                standalone.push(pair.vfs)
            } else {
                dependants.push(pair);
            }
        }
    );  

    let graph = DependencyGraph::from(dependants.as_slice());

    let mut resolved: Vec<Arc<dyn VirtualFS>> = Vec::new();
    let unresolved = graph.unresolved_dependencies().cloned().collect::<Vec<_>>();

    graph.into_iter().for_each(|entry| {
        if let Step::Resolved(pair) = entry {
            // If none of the dependencies are unresolved, we keep the mod
            if pair.config.dependencies.iter().filter(|dep| unresolved.contains(dep)).count() == 0 {
                resolved.push(pair.vfs.clone());
            } else {
                // If any of the dependencies are unresolved, we discard the mod and signal to the next ones that this mod is missing
                for dep in &pair.config.dependencies {
                    if unresolved.contains(dep) {
                        panic!("Mod '{}' requires mod dependency '{}' but it is missing.\n\nMake sure you have followed the installation instructions properly.", pair.config.name, dep);
                    }
                }
            }
        }
    });
    
    // Add the mods with no configuration last
    resolved.extend(standalone);

    let mut interner = HashedPathInterner::default();

    // Start the list with the root path
    let mut builder = FilesystemBuilder::new();

    let hash_to_index = resolved.iter().enumerate().flat_map(|(idx, modpack)| {

        modpack.discover().iter().map(|path| {
            builder.add_file(path);

            interner.add(hash(path.as_path()) as u64, path);
            (hash(path.as_path()) as u32, idx)
        }).collect::<Vec<_>>()
    }).collect::<MultiMap<u32, usize>>();
        
    // let hash_to_index = mods.iter().enumerate().flat_map(|(idx, modpack)| {
    //     modpack.discover().iter().map(|path| {
    //         builder.add_file(path);

    //         interner.add(hash(path.as_path()) as u64, path);
    //         (hash(path.as_path()) as u32, idx)
    //     }).collect::<Vec<_>>()
    // }).collect::<MultiMap<u32, usize>>();

    // dbg!(&builder.paths);
    let (paths, dir_infos) = builder.finish();

    // Next, we need to build a table of the hashes for the relative path being tied to the locator
    
    Manager {
        vfs: resolved,
        interner,
        lookup: hash_to_index,
        paths,
        dir_infos,
    }
});


impl Manager {
    pub fn get() -> &'static Manager {
        &MANAGER
    }

    fn transform_key(key: impl AsRef<str>) -> String {
        // We want the keys to be lowercased for Cobalt
        key.as_ref().to_string()
    }

    fn get_provider(&self) {

    }
    
    pub fn get_directory(&self, path: impl AsRef<Utf8Path>) -> Result<&DirectoryInfo, ModError> {
        if let Ok(index) = self.dir_infos.binary_search_by_key(&hash(&path), |dirinfo| dirinfo.path.hash) {
            Ok(&self.dir_infos[index])
        } else {
            Err(ModError::MissingDirectoryu32(hash(path)))
        }
    }

    pub fn get_path_by_hash(&self, hash: u32) -> Result<&ResourcePath, ModError> {
        if let Ok(index) = self.paths.binary_search_by_key(&hash, |resource| resource.path.hash) {
            Ok(&self.paths[index])
        } else {
            Err(ModError::MissingDirectoryu32(hash))
        }
    }

    pub fn get_directory_by_hash(&self, hash: u32) -> Result<&DirectoryInfo, ModError> {
        if let Ok(index) = self.dir_infos.binary_search_by_key(&hash, |dirinfo| dirinfo.path.hash) {
            Ok(&self.dir_infos[index])
        } else {
            Err(ModError::MissingDirectoryu32(hash))
        }
    }

    pub fn get_parent_directory(&self, dirinfo: &DirectoryInfo) -> Result<&DirectoryInfo, ModError> {
        if let Ok(index) = self.dir_infos.binary_search_by_key(&dirinfo.parent, |dirinfo| dirinfo.path.hash) {
            Ok(&self.dir_infos[index])
        } else {
            Err(ModError::MissingDirectoryu32(dirinfo.parent))
        }
    }

    pub fn get_files_in_directory(&self, dirinfo: &DirectoryInfo) -> Result<Vec<Utf8PathBuf>, ModError> {
        self.get_paths_in_directory(dirinfo)
            .map(|path| self.interner.try_get(path.path.hash).ok_or(ModError::MissingFile))
            .collect()
    }

    pub fn get_files_in_directory_and_subdir(&self, dirinfo: &DirectoryInfo) -> Result<Vec<Utf8PathBuf>, ModError> {
        self.get_paths_in_directory_and_subdir(dirinfo)
            .map(|path| self.interner.try_get(path.path.hash).ok_or(ModError::MissingFile))
            .collect()
    }

    fn get_paths_in_directory<'a>(&'a self, dirinfo: &'a DirectoryInfo) -> impl Iterator<Item = &'a ResourcePath> {
        dirinfo.file_hashes.iter().map(|hash| self.get_path_by_hash(*hash).unwrap())
    }

    fn get_paths_in_directory_and_subdir<'a>(&'a self, dirinfo: &'a DirectoryInfo) -> impl Iterator<Item = &'a ResourcePath> {
        self.get_child_directories_and_subdirs(dirinfo).flat_map(|dir| {
            self.get_paths_in_directory(dir)
        })
    }

    /// Look up the DirectoryInfo entries that are children of the provided one
    fn get_child_directories<'a>(&'a self, dirinfo: &'a DirectoryInfo) -> impl Iterator<Item = &'a DirectoryInfo> {
        dirinfo.child_dir_hashes.iter().map(|hash| {
            self.get_directory_by_hash(*hash).unwrap()
        })
    }

    fn get_child_directories_and_subdirs<'a>(&'a self, dirinfo: &'a DirectoryInfo) -> DirectoryInfoHierarchyIterator<'a> {
        DirectoryInfoHierarchyIterator::new(self, dirinfo)
    }

    pub fn get_full_path(&self, key: impl AsRef<Utf8Path>) -> Result<Utf8PathBuf, ModError> {
        let hash = hash(key);

        if let Ok(index) = self.paths.binary_search_by_key(&hash, |path| path.path.hash) {
                self.interner.try_get(self.paths[index].path.hash).ok_or(ModError::MissingFile)
        } else {
            Err(ModError::MissingFile)
        }
    }

    pub fn get_absolute_full_path(&self, key: impl AsRef<Utf8Path>) -> Result<Utf8PathBuf, ModError> {
        let key = key.as_ref();

        let hash = hash(key);

        let index = self.lookup.get_vec(&hash).ok_or(ModError::MissingFile)?;

        let root = self.vfs[index[0]].get_root();
        Ok(root.to_path_buf().join(key))
    }

    pub fn get_full_path_original(&self, key: impl AsRef<str>) -> Result<Utf8PathBuf, ModError> {
        self.interner.try_get(hash(key.as_ref())).ok_or(ModError::MissingFile)
    }

    pub fn exists(&self, key: impl AsRef<Utf8Path>) -> bool {
        let key = key.as_ref();
        let hash = hash(key);

        self.lookup.get_vec(&hash).is_some()
    }

    pub fn get_last_modified(&self, key: impl AsRef<Utf8Path>) -> Result<u64, ModError> {
        let key = key.as_ref();
        let hash = hash(key);

        let index = self.lookup.get_vec(&hash).ok_or(ModError::MissingFile)?;

        self.vfs[index[0]].last_modified(key)
    }

    pub fn get_file(&self, key: impl AsRef<Utf8Path>) -> Result<Vec<u8>, ModError> {
        let key = key.as_ref();

        let hash = hash(key);

        let index = self.lookup.get_vec(&hash).ok_or(ModError::MissingFile)?;

        self.vfs[index[0]].load(key)
    }

    pub fn get_files(&self, key: impl AsRef<Utf8Path>) -> Result<Vec<Vec<u8>>, ModError> {
        let key = key.as_ref();

        let hash = hash(key);

        self.lookup.get_vec(&hash).ok_or(ModError::MissingFile)?.iter().map(|idx| self.vfs[*idx].load(key)).collect()
    }

    pub fn get_files_with_locations(&self, key: impl AsRef<Utf8Path>) -> Result<Vec<(Vec<u8>, &Utf8Path)>, ModError> {
        let hash = hash(key);

        let path = self.interner.try_get(hash).ok_or(ModError::MissingFile)?;

        self.lookup.get_vec(&hash)
            .ok_or(ModError::MissingFile)?
            .iter()
            .map(|idx| { 
                self.vfs[*idx].load(&path)
                .map(|file| (file, self.vfs[*idx].get_root()))
            })
            .collect()
    }

    pub fn get_locations(&self) -> impl Iterator<Item = Utf8PathBuf> + '_ {
        self.interner.paths()
    }
}

struct DirectoryInfoHierarchyIterator<'a> {
    manager: &'a Manager,
    directories: Vec<&'a DirectoryInfo>,
}

impl<'a> DirectoryInfoHierarchyIterator<'a> {
    pub fn new(manager: &'a Manager, dir_info: &'a DirectoryInfo) -> Self {
        Self {
            manager,
            directories: vec![dir_info]
        }

    }
}

impl<'a> Iterator for DirectoryInfoHierarchyIterator<'a> {
    type Item = &'a DirectoryInfo;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(dir_info) = self.directories.pop() {
            // Get child directories by hash and push them to be queried later
            self.directories.extend(self.manager.get_child_directories(dir_info));

            Some(dir_info)
        } else {
            None
        }
    }
}