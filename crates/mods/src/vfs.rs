use std::{fs::File, io::BufReader, sync::RwLock};

use camino::{Utf8PathBuf, Utf8Path};
use walkdir::WalkDir;
use zip::{result::ZipError, ZipArchive};

use crate::{manager::ModConfig, ModError};


pub trait VirtualFS: Sync + Send {
    fn get_root(&self) -> &Utf8Path;
    fn discover(&self) -> Vec<Utf8PathBuf>;
    fn last_modified(&self, relative_path: &Utf8Path) -> Result<u64, ModError>;
    fn load(&self, relative_path: &Utf8Path) -> Result<Vec<u8>, ModError>;
}

impl dyn VirtualFS {
    pub fn get_config(&self) -> Result<ModConfig, ModError> {
        self.load("config.yaml".into()).map(|test| serde_yaml::from_slice(&test).map_err(ModError::ConfigError))?
    }
}

pub struct ModDir  {
    root: Utf8PathBuf,
}

impl ModDir {
    pub fn new(root: impl AsRef<Utf8Path>) -> Self {
        let out = Utf8PathBuf::from(root.as_ref());
        
        Self {
            root: out,
        }
    }
}

impl VirtualFS for ModDir {
    fn discover(&self) -> Vec<Utf8PathBuf> {
        WalkDir::new(&self.root)
            .into_iter()
            .flatten()
            .filter(|entry| entry.file_type().is_file() && entry.path().extension().is_some())
            .map(move |entry| {
                // Ignore the directories, only care about the files that have an extension
                Utf8PathBuf::from_path_buf(entry.path().strip_prefix(&self.root).unwrap().into()).unwrap()
            }).collect()
    }

    fn load(&self, relative_path: &Utf8Path) -> Result<Vec<u8>, ModError> {
        let full_path = self.root.join(relative_path);
        std::fs::read(full_path).map_err(ModError::IoError)
    }

    fn last_modified(&self, relative_path: &Utf8Path) -> Result<u64, ModError> {
        let full_path = self.root.join(relative_path);
        let mut timestamp = nnsdk::fs::FileTimeStamp::new();
        let filepath = std::ffi::CString::new(full_path.to_string()).unwrap();
        unsafe { nnsdk::fs::GetFileTimeStampForDebug(&mut timestamp, filepath.as_c_str().to_bytes_with_nul().as_ptr()) };

        Ok(timestamp.modify.time)
    }

    fn get_root(&self) -> &Utf8Path {
        &self.root
    }
}

pub struct ZippedMod {
    root: Utf8PathBuf,
    file: RwLock<ZipArchive<BufReader<File>>>,
}

impl ZippedMod {
    pub fn new(root: impl AsRef<Utf8Path>) -> Self {
        let out = root.as_ref().into();
        let file = std::io::BufReader::new(std::fs::File::open(&out).unwrap());
        let arc = match zip::ZipArchive::new(file) {
            Ok(arc) => arc,
            Err(err) => {
                let message = match err {
                    ZipError::Io(error) => &error.to_string(),
                    ZipError::InvalidArchive(err) => &format!("the file is malformed: {}", err),
                    ZipError::UnsupportedArchive(_) => "the compression or format of this file is unsupported",
                    ZipError::FileNotFound => unreachable!(),
                };
                panic!("Could not read zipped mod '{}' because {}", out, message);
            },
        };
        
        Self {
            root: out,
            file: RwLock::new(arc),
        }
    }
}

impl VirtualFS for ZippedMod {
    fn discover(&self) -> Vec<Utf8PathBuf> {
        let archive: Vec<Utf8PathBuf> = self.file.read().unwrap().file_names().map(Utf8Path::new).filter(|path| path.extension().is_some() && !path.starts_with("__MACOSX")).map(Utf8PathBuf::from).collect();

        archive
    }
    
    fn load(&self, relative_path: &Utf8Path) -> Result<Vec<u8>, ModError> {
        let mut file = self.file.write().unwrap();
        let mut arc = file.by_name(relative_path.as_str()).map_err(|err| ModError::IoError(err.into()))?;
        let mut out_buf = Vec::with_capacity(arc.size() as usize);
        std::io::copy(&mut arc, &mut out_buf).unwrap();
        Ok(out_buf)
    }

    fn last_modified(&self, relative_path: &Utf8Path) -> Result<u64, ModError> {
        let mut file = self.file.write().unwrap();
        let arc = file.by_name(relative_path.as_str()).map_err(|err| ModError::IoError(err.into()))?;
        let datatime = arc.last_modified();
        Ok((datatime.datepart() + datatime.timepart()) as u64)
    }

    fn get_root(&self) -> &Utf8Path {
        &self.root
    }
}