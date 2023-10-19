use super::Position;
use super::{ TextEditTrait, TextFile };
use std::fmt::Debug;
use std::path::{Path, PathBuf};

#[derive(Clone, PartialEq )]
pub struct SourceFile {
    pub file_id: u64,
    pub file: PathBuf,
    pub source: TextFile,
}

impl SourceFile {
    pub fn new<P: AsRef<Path>>(file: P, source: &str, file_id : u64) -> Self {
        Self {
            file: file.as_ref().to_path_buf(),
            source: TextFile::new(source),
            file_id
        }
    }

    pub fn as_text_edit(&mut self) -> &TextFile {
        &self.source
    }

    /// Get Line n from source file
    /// LINE starts at zero, must be adjusted for position
    pub fn get_line(&self, line: usize) -> Option<&str> {
        self.source.get_line(line).ok()
    }

    pub fn get_span(&self, p: &Position) -> &str {
        // If the span is zero in length then return the single char at that position
        let range = if p.range.is_empty() {
            p.range.start..p.range.start + 1
        } else {
            p.range.clone()
        };
        &self.source.source[range]
    }
}

impl Debug for SourceFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut x = f.debug_struct("SourceFile");
        x.field("file", &self.file.to_string_lossy());
        x.finish()
    }
}
