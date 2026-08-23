#![deny(unused_imports)]
use super::{AsmSource, Position, SResult, SourceErrorType, SourceFile, SourceInfo};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct SourceFiles {
    pub id_to_source_file: HashMap<u64, SourceFile>,
    pub path_to_id: HashMap<PathBuf, u64>,
    base_dir: PathBuf,
}

impl Default for SourceFiles {
    fn default() -> Self {
        Self {
            id_to_source_file: HashMap::new(),
            path_to_id: HashMap::new(),
            base_dir: std::env::current_dir().unwrap_or_default(),
        }
    }
}

impl SourceFiles {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a source set whose relative paths are resolved from `base_dir`.
    pub fn with_base_dir<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            base_dir: grl_utils::fileutils::abs_path(
                base_dir,
                std::env::current_dir().unwrap_or_default(),
            ),
            ..Self::default()
        }
    }

    /// Change the root used to normalize future source paths.
    pub fn set_base_dir<P: AsRef<Path>>(&mut self, base_dir: P) {
        self.base_dir =
            grl_utils::fileutils::abs_path(base_dir, std::env::current_dir().unwrap_or_default());
    }

    fn normalize_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        grl_utils::fileutils::abs_path(path, &self.base_dir)
    }

    fn get_next_id(&self) -> u64 {
        let max = self.id_to_source_file.keys().max();
        max.map(|x| x + 1).unwrap_or(0)
    }

    pub fn add_source_file<P: AsRef<Path>>(&mut self, p: P, text: &str) -> u64 {
        let p = self.normalize_path(p);

        // A source path identifies one logical source file in a project.  Keep
        // its file id stable when the same file is encountered through another
        // include or assembly target; positions stored in ASTs and diagnostics
        // refer to this id.
        if let Some(id) = self.path_to_id.get(&p) {
            return *id;
        }

        let id = self.get_next_id();
        let source_file = SourceFile::new(&p, text, AsmSource::FileId(id));
        self.id_to_source_file.insert(id, source_file);
        self.path_to_id.insert(p, id);
        id
    }

    /// Replace a source snapshot while preserving its project-wide file id.
    ///
    /// Existing ASTs should be considered stale after replacement, but source
    /// positions remain meaningful because they continue to identify the same
    /// logical file.
    pub fn replace_source_file<P: AsRef<Path>>(&mut self, p: P, text: &str) -> SResult<u64> {
        let p = self.normalize_path(p);
        let id = *self
            .path_to_id
            .get(&p)
            .ok_or_else(|| SourceErrorType::FileNotFound(p.to_string_lossy().into()))?;
        self.id_to_source_file
            .insert(id, SourceFile::new(&p, text, AsmSource::FileId(id)));
        Ok(id)
    }

    pub fn get_hash<P: AsRef<Path>>(&self, p: P) -> SResult<String> {
        self.get_source(&p)
            .map(|(_, src)| src.get_text().get_hash())
    }

    pub fn get_source<P: AsRef<Path>>(&self, p: P) -> SResult<(u64, &SourceFile)> {
        let p = self.normalize_path(p);

        let id = self
            .path_to_id
            .get(&p)
            .ok_or_else(|| SourceErrorType::FileNotFound(p.to_string_lossy().into()))?;
        self.id_to_source_file
            .get(id)
            .map(|source| (*id, source))
            .ok_or(SourceErrorType::IdNotFound(*id))
    }

    pub fn get_source_file_from_id(&self, id: u64) -> SResult<&SourceFile> {
        self.id_to_source_file
            .get(&id)
            .ok_or(SourceErrorType::IdNotFound(id))
    }

    pub fn get_source_mut<P: AsRef<Path>>(&mut self, p: P) -> SResult<&mut SourceFile> {
        let p = self.normalize_path(p);

        let id = self
            .path_to_id
            .get(&p)
            .ok_or_else(|| SourceErrorType::FileNotFound(p.to_string_lossy().into()))?;
        self.id_to_source_file
            .get_mut(id)
            .ok_or(SourceErrorType::IdNotFound(*id))
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
            let fragment = source_file.try_get_span(pos).ok_or_else(|| {
                SourceErrorType::InvalidSourceRange {
                    file: source_file.file.to_string_lossy().into_owned(),
                    range: pos.range(),
                }
            })?;
            let line_str =
                source_file
                    .get_line(pos.line())
                    .ok_or_else(|| SourceErrorType::LineNotFound {
                        file: source_file.file.to_string_lossy().into_owned(),
                        line: pos.line(),
                    })?;

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

#[cfg(test)]
mod tests {
    use super::SourceFiles;
    use crate::{AsmSource, Position, SourceErrorType};

    #[test]
    fn adding_a_path_twice_preserves_identity() {
        let mut files = SourceFiles::new();
        let first = files.add_source_file("main.asm", "one");
        let second = files.add_source_file("main.asm", "two");

        assert_eq!(first, second);
        assert_eq!(
            files.get_source("main.asm").unwrap().1.get_entire_source(),
            "one"
        );
    }

    #[test]
    fn replacing_a_snapshot_preserves_identity() {
        let mut files = SourceFiles::new();
        let id = files.add_source_file("main.asm", "one");

        assert_eq!(files.replace_source_file("main.asm", "two").unwrap(), id);
        let (new_id, source) = files.get_source("main.asm").unwrap();
        assert_eq!(new_id, id);
        assert_eq!(source.get_entire_source(), "two");
    }

    #[test]
    fn invalid_source_ranges_are_reported() {
        let mut files = SourceFiles::new();
        let id = files.add_source_file("main.asm", "one");
        let position = Position::new(0, 0, 99..100, AsmSource::FileId(id));

        assert!(matches!(
            files.get_source_info(&position),
            Err(SourceErrorType::InvalidSourceRange { .. })
        ));
    }

    #[test]
    fn relative_and_absolute_paths_share_an_identity() {
        let base = std::env::current_dir().unwrap();
        let mut files = SourceFiles::with_base_dir(&base);
        let relative = files.add_source_file("main.asm", "one");
        let absolute = files.add_source_file(base.join("main.asm"), "two");

        assert_eq!(relative, absolute);
        assert_eq!(files.path_to_id.len(), 1);
    }
}
