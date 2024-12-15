use camino::Utf8Path;

use crate::{manager::{ResourcePath, DirectoryInfo}, hash};

#[derive(Debug, Default)]
pub struct FilesystemBuilder {
    paths: Vec<ResourcePath>,
    dir_infos: Vec<DirectoryInfo>,
}

impl FilesystemBuilder {
    pub fn new() -> Self {
        let mut instance = Self::default();
        let resource = instance.add_resource_path(ResourcePath::new_from_path(Utf8Path::new("")));
        let dirinfo = DirectoryInfo::new_from_resourcepath(resource);
        instance.dir_infos.push(dirinfo);
        instance
    }

    pub fn finish(mut self) -> (Vec<ResourcePath>, Vec<DirectoryInfo>) {
        self.paths.sort_by_key(|k| k.path.hash);
        self.dir_infos.sort_by_key(|k| k.path.hash);
        (self.paths, self.dir_infos)
    }

    pub fn get_folder_by_hash(&self, hash: u32) -> Option<&DirectoryInfo> {
        self.dir_infos.iter().find(|folder| folder.path.hash == hash)
    }
    
    pub fn get_folder_by_hash_mut(&mut self, hash: u32) -> Option<&mut DirectoryInfo> {
        self.dir_infos.iter_mut().find(|folder| folder.path.hash == hash)
    }

    /// Add a ResourcePath to the list, and return a reference to it
    fn add_resource_path(&mut self, mut resource_path: ResourcePath) -> &ResourcePath {
        resource_path.path.index = self.paths.len() as u32;
        self.paths.push(resource_path);
        self.paths.last().unwrap()
    }

    /// Add a ResourcePath to the list, and return a reference to it
    fn add_parented_resource_path(&mut self, mut resource_path: ResourcePath) -> ResourcePath {
        let parent_dir = self.get_folder_by_hash_mut(resource_path.parent.hash).unwrap_or_else(|| panic!("couldn't get parent for ResourcePath {:?}", resource_path));
        // Set the ResourcePath's parent
        resource_path.parent = parent_dir.path;
        // Add the new directory as a child of the parent directory
        parent_dir.child_dir_hashes.push(resource_path.path.hash);

        *self.add_resource_path(resource_path)
    }

    /// Add a file in the ResourcePath table and relevant directory
    /// 
    /// If the directory is missing, it will be recursively created along with its parents.
    pub fn add_file(&mut self, path: impl AsRef<Utf8Path>) {
        let path = path.as_ref();

        let parent = path.parent().unwrap_or("".into());

        // Make sure the parent directory exists or create it recursively
        let parent_dir = match self.get_folder_by_hash_mut(hash(parent)) {
            Some(parent) => parent,
            None => {
                self.add_folder_recursive(parent);
                self.get_folder_by_hash_mut(hash(parent)).unwrap()
            },
        };
        
        let mut resource_path = ResourcePath::new_from_path(path);

        // Set the ResourcePath's parent
        resource_path.parent = parent_dir.path;
        // Add the new directory as a child of the parent directory
        parent_dir.file_hashes.push(resource_path.path.hash);
        self.add_resource_path(resource_path);
    }

    /// Create a directory and attach it to the parent directory, if it exists.
    pub fn add_directory(&mut self, path: impl AsRef<Utf8Path>) -> &DirectoryInfo {
        let path = path.as_ref();

        let new_path = ResourcePath::new_from_path(path);
        let resource_path = self.add_parented_resource_path(new_path);

        // println!("AddDirectoryInfo: path: {}", path);
        // println!("AddDirectoryInfo: parent's child directories: {:?}", parent_dir.child_dir_hashes);

        // Add a path for the directory
        self.add_directory_info(&resource_path)
    }

    pub fn add_directory_info(&mut self, new_path: &ResourcePath) -> &DirectoryInfo {
        let resource_path = self.add_resource_path(*new_path);

        let new_directory = DirectoryInfo::new_from_resourcepath(resource_path);
        self.dir_infos.push(new_directory);
        self.dir_infos.last().unwrap()
    }
    
    /// Create the directory and its parents if they do not exist.
    pub fn add_folder_recursive(&mut self, path: impl AsRef<Utf8Path>) {
        let path = path.as_ref();

        // Check if the directory already exists
        if self.get_folder_by_hash(hash(path)).is_none() {
            self.create_folder_hierarchy(path);
        }
    }
    
    /// Add a directory to the filesystem if its parent exists, and create the parent if it does not by calling itself recursively
    pub fn create_folder_hierarchy(&mut self, path: impl AsRef<Utf8Path>) -> &DirectoryInfo {
        let path = path.as_ref();
    
        let parent = path.parent().expect("should be able to get parent");
    
        // Check if the parent already exists
        if self.get_folder_by_hash(hash(parent)).is_none() {
            // Parent is missing, recursively call to create the parent
            self.create_folder_hierarchy(parent);
        }

        // The parent has been created by now
        self.add_directory(path)
    }
}