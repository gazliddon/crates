use super::{SourceFiles, SourceFile};
use utils::{PathSearcher, Paths, SearchError};

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub struct SourceFileLoader {
    pub source_search_paths: Paths,
    pub sources: SourceFiles,
    id: u64,
    pub files_loaded: HashSet<PathBuf>,
    pub files_written: HashSet<PathBuf>,
}

impl Default for SourceFileLoader {
    fn default() -> Self {
        let vec: Vec<&str> = vec!["."];
        Self::from_search_paths(&vec)
    }
}

pub trait FileIo: PathSearcher {
    fn mk_error(&self, e: utils::SearchError) -> anyhow::Error {
        use utils::SearchError::*;

        match e {
            FileNotFound(f, v) => {
                let errs: Vec<String> = v
                    .into_iter()
                    .map(|f| f.to_string_lossy().to_string())
                    .collect();
                anyhow!(
                    "Can't load {}\n Tried:\n{}",
                    f.to_string_lossy(),
                    errs.join("\n")
                )
            }
            _ => {
                panic!()
            }
        }
    }

    fn add_to_files_read(&mut self, p: PathBuf);
    fn add_to_files_written(&mut self, p: PathBuf);

    fn expand_path<P: AsRef<Path>>(&self, p: P) -> PathBuf {
        p.as_ref().to_path_buf()
    }

    fn get_files_written(&self) -> Vec<PathBuf>;
    fn get_files_read(&self) -> Vec<PathBuf>;

    fn read_to_string<P: AsRef<Path>>(&mut self, path: P) -> Result<(PathBuf, String)> {
        let path = self
            .get_full_path(path.as_ref())
            .map_err(|e| self.mk_error(e))?;

        let ret = fs::read_to_string(path.clone())?;
        let abs_path = utils::fileutils::abs_path_from_cwd(&path);
        self.add_to_files_read(abs_path);
        Ok((path, ret))
    }

    fn read_binary<P: AsRef<Path>>(&mut self, path: P) -> Result<(PathBuf, Vec<u8>)> {
        use std::fs::File;
        use std::io::Read;
        let mut buffer = vec![];

        let path = self.expand_path(path);

        let path = self
            .get_full_path(path)
            .map_err(|e| self.mk_error(e))?;

        let mut file = File::open(path.clone())?;
        file.read_to_end(&mut buffer)?;
        self.add_to_files_read(path.clone());
        Ok((path, buffer))
    }

    fn write<P: AsRef<Path>, C: AsRef<[u8]>>(&mut self, path: P, data: C) -> PathBuf {
        let path = self.expand_path(path);

        std::fs::write(&path, data).unwrap_or_else(|_| 
            panic!(
             "Can't write bin file {}", path.to_string_lossy() 
                ));

        let abs_path = utils::fileutils::abs_path_from_cwd(&path);

        self.add_to_files_written(abs_path);

        path
    }

    fn read_binary_chunk<P: AsRef<Path>>(
        &mut self,
        path: P,
        r: std::ops::Range<usize>,
    ) -> Result<(PathBuf, Vec<u8>)> {
        let (path, buff) = self.read_binary(path)?;

        let buff_r = 0..buff.len();

        let start = r.start;
        let last = (r.len() + r.start) - 1;

        if buff_r.contains(&start) && buff_r.contains(&last) {
            Ok((path, buff[r].into()))
        } else {
            Err(anyhow!(
                "Cannot read binary chunk. Range exceeds size of file {}: file size is {}, tried to grab up to {}",
                path.to_string_lossy(), buff_r.len(), last
                ))
        }
    }

    fn get_size<P: AsRef<Path>>(&self, path: P) -> Result<usize> {
        let path = self.expand_path(path);
        let path = self
            .get_full_path(path)
            .map_err(|e| self.mk_error(e))?;

        let md = std::fs::metadata(path.clone())
            .map_err(|e| anyhow!("Can't get size for : {}\n{}", path.to_string_lossy(), e))?;

        Ok(md.len() as usize)
    }
}

impl PathSearcher for SourceFileLoader {
    fn get_full_path<P: AsRef<Path>>(&self, file: P) -> Result<PathBuf, SearchError> {
        self.source_search_paths.get_full_path(file)
    }

    fn get_search_paths(&self) -> &[ PathBuf ] {
        self.source_search_paths.get_search_paths()
    }

    fn add_search_path<P: AsRef<Path>>(&mut self, path : P) {
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
}

impl SourceFileLoader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read_source<P: AsRef<Path>>(&mut self, path: P) -> Result<&SourceFile> {
        let (path, text) = self.read_to_string(path)?;
        self.add_source_file(&path, &text)
    }
    pub fn add_source_file<P: AsRef<Path>>(&mut self, path: P, text: &str) -> Result<&SourceFile> {
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
        }
    }
}
