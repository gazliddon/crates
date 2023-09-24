use std::path::{Path, PathBuf};
use thin_vec::{ ThinVec, thin_vec };

use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum SearchError {
    FileNotFound(Box<PathBuf>, ThinVec<PathBuf>),
    Placeholder,
}

impl std::fmt::Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub trait PathSearcher {
    fn get_full_path<P: AsRef<Path>>(&self, file: P) -> Result<PathBuf, SearchError>;
    fn get_search_paths(&self) -> &[PathBuf];
    fn add_search_path<P: AsRef<Path>>(&mut self, path : P);
    fn set_search_paths(&mut self, paths: &[PathBuf]);
}

#[derive(Debug, Clone, Default)]
pub struct Paths {
    paths: Vec<PathBuf>,
}

impl Paths {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn from_paths<P: AsRef<Path>>(paths: &[P]) -> Self {
        let paths = paths.iter().map(|p| p.as_ref().into()).collect();
        let mut ret = Self::new();
        ret.paths = paths;
        ret
    }

    pub fn add_path<P: AsRef<Path>>(&mut self, path: P) {
        self.paths.push(path.as_ref().to_path_buf())
    }
}

impl PathSearcher for Paths {
    fn get_search_paths(&self) -> &[ PathBuf ] {
        &self.paths
    }

    fn add_search_path<P: AsRef<Path>>(&mut self, path: P) {
        self.paths.push(path.as_ref().to_path_buf())
    }

    fn set_search_paths(&mut self, paths: &[PathBuf]) {
        self.paths = paths.into()
    }

    fn get_full_path<P: AsRef<Path>>(&self, file_name: P) -> Result<PathBuf, SearchError> {
        let file_name = file_name.as_ref();
        let mut tried: ThinVec<_> = thin_vec![file_name.to_path_buf()];

        if !file_name.has_root() {
            for i in &self.paths {
                let x = i.join(file_name);
                if x.exists() {
                    let ret = crate::fileutils::abs_path_from_cwd(x);
                    return Ok(ret);
                } else {
                    tried.push(x.clone());
                }
            }
        } else if file_name.exists() {
            return Ok(file_name.to_path_buf());
        }

        Err(SearchError::FileNotFound(file_name.to_path_buf().into(), tried))
    }
}
