#![deny(unused_imports)]

use super::{SourceFile, SourceFiles};
use grl_utils::{PathSearcher, Paths, FResult,FileIo};


use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct SourceFileLoader {
    pub source_search_paths: Paths,
    pub sources: SourceFiles,
    id: u64,
    pub files_loaded: HashSet<PathBuf>,
    pub files_written: HashSet<PathBuf>,
    bin_file_cache: HashMap<PathBuf, Vec<u8>>,
}

impl Default for SourceFileLoader {
    fn default() -> Self {
        let vec: Vec<&str> = vec!["."];
        Self::from_search_paths(&vec)
    }
}

impl PathSearcher for SourceFileLoader {
    fn get_full_path<P: AsRef<Path>>(&self, file: P) -> FResult<PathBuf> {
        self.source_search_paths.get_full_path(file)
    }

    fn get_search_paths(&self) -> &[PathBuf] {
        self.source_search_paths.get_search_paths()
    }

    fn add_search_path<P: AsRef<Path>>(&mut self, path: P) {
        self.source_search_paths.add_path(path)
    }

    fn set_search_paths(&mut self, paths: &[PathBuf]) {
        self.source_search_paths.set_search_paths(paths)
    }
}

impl FileIo for SourceFileLoader {
    fn get_files_written(&self) -> Vec<PathBuf> {
        self.files_written.iter().cloned().collect()
    }

    fn get_files_read(&self) -> Vec<PathBuf> {
        self.files_loaded.iter().cloned().collect()
    }

    fn add_to_files_read(&mut self, path: PathBuf) {
        self.files_loaded.insert(path);
    }

    fn add_to_files_written(&mut self, path: PathBuf) {
        self.files_written.insert(path);
    }

    fn read_binary<P: AsRef<Path>>(&mut self, path: P) -> FResult<(PathBuf, Vec<u8>)> {
        let path = self.get_full_path(path)?;

        if let Some(cached) = self.bin_file_cache.get(&path) {
            Ok((path, cached.clone()))
        } else {
            let mut buffer = vec![];
            let mut file = File::open(path.clone())?;
            file.read_to_end(&mut buffer)?;
            self.add_to_files_read(path.clone());
            self.bin_file_cache.insert(path.clone(), buffer.clone());
            Ok((path, buffer))
        }
    }
}

impl SourceFileLoader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read_source<P: AsRef<Path>>(&mut self, path: P) -> FResult<&SourceFile> {
        let (path, text) = self.read_to_string(path)?;
        self.add_source_file(&path, &text)
    }

    pub fn add_source_file<P: AsRef<Path>>(&mut self, path: P, text: &str) -> FResult<&SourceFile> {
        let id = self.sources.add_source_file(&path, text);
        let sf = self.sources.get_source_file_from_id(id).unwrap();
        Ok(sf)
    }

    pub fn from_search_paths<P: AsRef<Path>>(paths: &[P]) -> Self {
        let search_paths: Vec<PathBuf> = paths.iter().map(|x| PathBuf::from(x.as_ref())).collect();
        Self {
            source_search_paths: Paths::from_paths(&search_paths),
            sources: SourceFiles::new(),
            id: 0,
            files_loaded: Default::default(),
            files_written: Default::default(),
            bin_file_cache: Default::default(),
        }
    }
}
