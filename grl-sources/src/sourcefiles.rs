#![deny(unused_imports)]
use super::{AsmSource, Position, SResult, SourceErrorType, SourceFile, SourceInfo};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Default)]
pub struct SourceFiles {
    pub id_to_source_file: HashMap<u64, SourceFile>,
    pub path_to_id: HashMap<PathBuf, u64>,
}

impl SourceFiles {
    pub fn new() -> Self {
        Self::default()
    }

    fn get_next_id(&self) -> u64 {
        let max = self.id_to_source_file.keys().max();
        max.map(|x| x + 1).unwrap_or(0)
    }

    pub fn add_source_file<P: AsRef<Path>>(&mut self, p: P, text: &str) -> u64 {
        let p = p.as_ref().to_path_buf();
        let id = self.get_next_id();
        let source_file = SourceFile::new(&p, text, AsmSource::FileId(id));
        self.id_to_source_file.insert(id, source_file);
        self.path_to_id.insert(p, id);
        id
    }

    pub fn get_hash<P: AsRef<Path>>(&self, p: P) -> SResult<String> {
        self.get_source(&p)
            .map(|(_, src)| src.get_text().get_hash())
    }

    pub fn get_source<P: AsRef<Path>>(&self, p: P) -> SResult<(u64, &SourceFile)> {
        let p = p.as_ref().to_path_buf();

        let id = self
            .path_to_id
            .get(&p)
            .ok_or_else(|| SourceErrorType::FileNotFound(p.to_string_lossy().into()))?;
        Ok((*id, self.id_to_source_file.get(id).unwrap()))
    }

    pub fn get_source_file_from_id(&self, id: u64) -> SResult<&SourceFile> {
        self.id_to_source_file
            .get(&id)
            .ok_or(SourceErrorType::IdNotFound(id))
    }

    pub fn get_source_mut<P: AsRef<Path>>(&mut self, p: P) -> SResult<&mut SourceFile> {
        let p = p.as_ref().to_path_buf();

        let id = self
            .path_to_id
            .get(&p)
            .ok_or_else(|| SourceErrorType::FileNotFound(p.to_string_lossy().into()))?;
        Ok(self.id_to_source_file.get_mut(id).unwrap())
    }

    pub fn remove_file<P: AsRef<Path>>(&mut self, p: P) -> SResult<()> {
        let p = p.as_ref().to_path_buf();

        let id = self
            .path_to_id
            .get(&p)
            .ok_or_else(|| SourceErrorType::FileNotFound(p.to_string_lossy().to_string()))?;
        self.remove_id(*id)
    }

    pub fn remove_id(&mut self, id: u64) -> SResult<()> {
        let sf = self
            .id_to_source_file
            .get(&id)
            .ok_or(SourceErrorType::IdNotFound(id))?;
        let p = sf.file.clone();
        self.path_to_id.remove(&p);
        self.id_to_source_file.remove(&id);
        Ok(())
    }

    pub fn get_source_info<'a>(&'a self, pos: &Position) -> SResult<SourceInfo<'a>> {
        if let AsmSource::FileId(file_id) = pos.src() {
            let source_file = self.get_source_file_from_id(file_id)?;
            let fragment = source_file.get_span(pos);
            let line_str = source_file.get_line(pos.line()).unwrap();

            let ret = SourceInfo {
                line_str,
                fragment,
                source_file,
                file: source_file.file.clone(),
                pos: *pos,
            };

            Ok(ret)
        } else {
            Err(SourceErrorType::Misc)
        }
    }
}
