use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use camino::{Utf8Path, Utf8PathBuf};
use dependency_graph::{DependencyGraph, Node, Step};
use serde::{Deserialize, Serialize};

use crate::{builder::FilesystemBuilder, discover_mods_manager, hash, vfs::VirtualFS, ModError};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) enum UpdateSource {
    #[serde(rename = "github")]
    GitHub { user: String, repo: String },
    #[serde(rename = "gamebanana")]
    GameBanana { item_type: String, item_id: u64 },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub(crate) enum Dependency {
    Id(String),
    Detailed {
        id: String,
        #[serde(default)]
        update_sources: Vec<UpdateSource>,
    },
}

impl Dependency {
    pub(crate) fn id(&self) -> &str {
        match self {
            Dependency::Id(id) => id,
            Dependency::Detailed { id, .. } => id,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub(crate) struct ModConfig {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) author: String,
    #[serde(default)]
    pub(crate) dependencies: Vec<Dependency>,
    #[serde(default)]
    pub(crate) optional_dependencies: Vec<Dependency>,
    pub(crate) source_url: Option<String>,
    #[serde(default)]
    pub(crate) update_sources: Vec<UpdateSource>,
}

pub struct Mod {
    pub(crate) vfs: Arc<dyn VirtualFS>,
    pub(crate) config: ModConfig,
}

impl Mod {
    pub fn id(&self) -> Option<&str> {
        (!self.config.id.is_empty()).then_some(&self.config.id)
    }

    pub fn name(&self) -> Option<&str> {
        (!self.config.name.is_empty()).then_some(&self.config.name)
    }

    pub fn root(&self) -> &Utf8Path {
        self.vfs.root()
    }

    pub fn is_writeable(&self) -> bool {
        self.vfs.is_writeable()
    }

    pub fn entries(&self) -> impl Iterator<Item = &Utf8Path> {
        self.vfs.entries().values().map(|p| p.as_path())
    }

    pub fn read(&self, rel: impl AsRef<Utf8Path>) -> Result<Vec<u8>, ModError> {
        self.vfs.load(hash(rel))
    }

    pub fn last_modified(&self, rel: impl AsRef<Utf8Path>) -> Result<u64, ModError> {
        self.vfs.last_modified(hash(rel))
    }

    pub fn write(&self, rel: impl AsRef<Utf8Path>, bytes: &[u8]) -> Result<(), ModError> {
        let rel = rel.as_ref();
        let target = self
            .vfs
            .entries()
            .get(&hash(rel))
            .cloned()
            .unwrap_or_else(|| rel.to_path_buf());
        self.vfs.write(&target, bytes)
    }
}

pub(crate) struct ModPair {
    config: ModConfig,
    vfs: Arc<dyn VirtualFS>,
    effective_deps: Vec<String>,
}

impl Node for ModPair {
    type DependencyType = String;

    fn dependencies(&self) -> &[Self::DependencyType] {
        &self.effective_deps
    }

    fn matches(&self, dependency: &Self::DependencyType) -> bool {
        self.config.id == *dependency
    }
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct DirectoryInfo {
    pub(crate) path: u32,
    pub(crate) file_hashes: Vec<u32>,
    pub(crate) child_dir_hashes: Vec<u32>,
}

impl DirectoryInfo {
    pub(crate) fn new(path: u32) -> Self {
        Self {
            path,
            ..Default::default()
        }
    }
}

pub struct Manager {
    mods: Vec<Mod>,
    lookup: HashMap<u32, Vec<usize>>,
    dir_infos: Vec<DirectoryInfo>,
}

static MANAGER: LazyLock<Manager> = LazyLock::new(|| Manager::from_root("sd:/engage/mods"));

impl Manager {
    pub fn get() -> &'static Manager {
        &MANAGER
    }

    pub(crate) fn from_root(root: impl AsRef<Utf8Path>) -> Self {
        let discovered = discover_mods_manager(root).collect::<Vec<_>>();

        let configs: Vec<(ModConfig, Arc<dyn VirtualFS>)> = discovered
            .iter()
            .map(|vfs| {
                let config = match vfs.config() {
                    Ok(conf) => conf,
                    Err(ModError::Config(_)) => {
                        panic!(
                            "Mod '{}' ran into a configuration error. Make sure the file is following the YAML specifications.",
                            vfs.root().file_name().unwrap()
                        );
                    },
                    Err(_) => ModConfig::default(),
                };

                (config, vfs.clone())
            })
            .collect();

        let present_ids: std::collections::HashSet<String> = configs
            .iter()
            .filter(|(config, _)| !config.id.is_empty())
            .map(|(config, _)| config.id.clone())
            .collect();

        let pairs: Vec<ModPair> = configs
            .into_iter()
            .map(|(config, vfs)| {
                let mut effective_deps: Vec<String> =
                    config.dependencies.iter().map(|dep| dep.id().to_string()).collect();
                effective_deps.extend(
                    config
                        .optional_dependencies
                        .iter()
                        .filter(|optional| present_ids.contains(optional.id()))
                        .map(|optional| optional.id().to_string()),
                );

                ModPair { config, vfs, effective_deps }
            })
            .collect();

        let mut dependants = Vec::new();
        let mut standalone = Vec::new();

        for pair in pairs {
            if pair.config.id.is_empty() {
                standalone.push(pair);
            } else {
                dependants.push(pair);
            }
        }

        let graph = DependencyGraph::from(dependants.as_slice());
        let unresolved = graph.unresolved_dependencies().cloned().collect::<Vec<_>>();

        let mut resolved: Vec<Mod> = Vec::new();

        for entry in graph.into_iter() {
            if let Step::Resolved(pair) = entry {
                if pair.config.dependencies.iter().all(|dep| !unresolved.iter().any(|u| u == dep.id())) {
                    resolved.push(Mod {
                        vfs: pair.vfs.clone(),
                        config: pair.config.clone(),
                    });
                } else {
                    for dep in &pair.config.dependencies {
                        if unresolved.iter().any(|u| u == dep.id()) {
                            panic!(
                                "Mod '{}' requires mod dependency '{}' but it is missing.\n\nMake sure you have followed the installation instructions properly.",
                                pair.config.name, dep.id()
                            );
                        }
                    }
                }
            }
        }

        resolved.extend(standalone.into_iter().map(|p| Mod { vfs: p.vfs, config: p.config }));

        let mut builder = FilesystemBuilder::new();
        let mut lookup: HashMap<u32, Vec<usize>> = HashMap::new();

        for (idx, modpack) in resolved.iter().enumerate() {
            for (&h, canonical) in modpack.vfs.entries() {
                builder.add_file(canonical, h);
                lookup.entry(h).or_default().push(idx);
            }
        }

        Manager {
            mods: resolved,
            lookup,
            dir_infos: builder.finish(),
        }
    }

    pub fn mods(&self) -> impl Iterator<Item = &Mod> {
        self.mods.iter()
    }

    pub fn read(&self, path: impl AsRef<Utf8Path>) -> Result<Vec<u8>, ModError> {
        let h = hash(path);
        let idx = self.first_provider(h).ok_or(ModError::NotFound)?;
        self.mods[idx].vfs.load(h)
    }

    pub fn read_layers(&self, path: impl AsRef<Utf8Path>) -> Result<Vec<Vec<u8>>, ModError> {
        let h = hash(path);
        let providers = self.lookup.get(&h).ok_or(ModError::NotFound)?;
        providers.iter().map(|idx| self.mods[*idx].vfs.load(h)).collect()
    }

    pub fn contains(&self, path: impl AsRef<Utf8Path>) -> bool {
        self.lookup.contains_key(&hash(path))
    }

    pub fn contains_directory(&self, path: impl AsRef<Utf8Path>) -> bool {
        self.directory(hash(path)).is_some()
    }

    pub fn last_modified(&self, path: impl AsRef<Utf8Path>) -> Result<u64, ModError> {
        let h = hash(path);
        let idx = self.first_provider(h).ok_or(ModError::NotFound)?;
        self.mods[idx].vfs.last_modified(h)
    }

    pub fn disk_path(&self, path: impl AsRef<Utf8Path>) -> Option<Utf8PathBuf> {
        let h = hash(path);
        let idx = self.first_provider(h)?;
        let vfs = &self.mods[idx].vfs;
        Some(vfs.root().join(vfs.canonical_path(h)?))
    }

    pub fn list<'a>(&'a self, dir: impl AsRef<Utf8Path>) -> impl Iterator<Item = &'a Utf8Path> + 'a {
        let dir_hash = hash(dir);
        self.directory(dir_hash)
            .into_iter()
            .flat_map(|d| d.file_hashes.iter().copied())
            .filter_map(move |h| self.canonical_for(h))
    }

    pub fn list_recursive<'a>(&'a self, dir: impl AsRef<Utf8Path>) -> impl Iterator<Item = &'a Utf8Path> + 'a {
        let root = hash(dir);
        let mut file_hashes: Vec<u32> = Vec::new();
        let mut stack = vec![root];
        while let Some(dir_hash) = stack.pop() {
            if let Some(d) = self.directory(dir_hash) {
                file_hashes.extend(&d.file_hashes);
                stack.extend(&d.child_dir_hashes);
            }
        }
        file_hashes.into_iter().filter_map(move |h| self.canonical_for(h))
    }

    fn first_provider(&self, hash: u32) -> Option<usize> {
        self.lookup.get(&hash).and_then(|v| v.first()).copied()
    }

    fn canonical_for(&self, hash: u32) -> Option<&Utf8Path> {
        let idx = self.first_provider(hash)?;
        self.mods[idx].vfs.canonical_path(hash)
    }

    fn directory(&self, hash: u32) -> Option<&DirectoryInfo> {
        self.dir_infos
            .binary_search_by_key(&hash, |dirinfo| dirinfo.path)
            .ok()
            .map(|index| &self.dir_infos[index])
    }
}
