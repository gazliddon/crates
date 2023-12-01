#![deny(unused_imports)]
use crate::{AsmSource, Position, TextEditTrait, TextFile};
use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

#[derive(Clone, PartialEq)]
pub struct SourceFile {
    pub file_id: AsmSource,
    pub file: PathBuf,
    source: TextFile,
}

impl SourceFile {
    pub fn get_entire_source(&self) -> &str {
        &self.source.source
    }
    pub fn new<P: AsRef<Path>>(file: P, source: &str, file_id: AsmSource) -> Self {
        Self {
            file: file.as_ref().to_path_buf(),
            source: TextFile::new(source),
            file_id,
        }
    }

    /// Get Line n from source file
    /// LINE starts at zero, must be adjusted for position
    pub fn get_line(&self, line: usize) -> Option<&str> {
        self.source.get_line(line).ok()
    }

    pub fn get_position(&self, r: std::ops::Range<usize>) -> Position {
        let tp = self.source.offset_to_text_pos(r.start).unwrap();
        Position::new(tp.line(), tp.col(), r, self.file_id)
    }

    pub fn get_span(&self, p: &Position) -> &str {
        let p_range = p.range();
        // If the span is zero in length then return the single char at that position
        let range = if p_range.is_empty() {
            p_range.start..p_range.start + 1
        } else {
            p_range
        };

        &self.source.source[range]
    }
    pub fn get_text(&self) -> &TextFile {
        &self.source
    }
    pub fn get_text_mut(&mut self) -> &mut TextFile {
        &mut self.source
    }
}

impl Debug for SourceFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut x = f.debug_struct("SourceFile");
        x.field("file", &self.file.to_string_lossy());
        x.finish()
    }
}
