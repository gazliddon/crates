#![deny(unused_imports)]
use crate::{AsmSource, Position, TextEditTrait, TextFile};
use std::{
    fmt::Debug,
    path::{Path, PathBuf},
    sync::Arc,
};

/// SourceFile is a wrapper around TextFile, it contains the file path and the file_id
#[derive(Clone, PartialEq)]
pub struct SourceFile {
    pub file_id: AsmSource,
    pub file: PathBuf,
    // Cloned source snapshots share their text. Editing uses copy-on-write.
    source: Arc<TextFile>,
}

impl SourceFile {
    pub fn get_entire_source(&self) -> &str {
        &self.source.source
    }
    pub fn new<P: AsRef<Path>>(file: P, source: &str, file_id: AsmSource) -> Self {
        Self {
            file: file.as_ref().to_path_buf(),
            source: Arc::new(TextFile::new(source)),
            file_id,
        }
    }

    /// Get Line n from source file
    /// LINE starts at zero, must be adjusted for position
    pub fn get_line(&self, line: usize) -> Option<&str> {
        self.source.get_line(line).ok()
    }

    pub fn get_position(&self, r: std::ops::Range<usize>) -> Position {
        self.try_get_position(r)
            .expect("source position must point into the source file")
    }

    /// Convert a byte range to a source position without panicking on an
    /// invalid offset or a non-character boundary.
    pub fn try_get_position(
        &self,
        r: std::ops::Range<usize>,
    ) -> Result<Position, crate::EditErrorKind> {
        if !self.source.source.is_char_boundary(r.start) {
            return Err(crate::EditErrorKind::NotCharBoundary(r.start));
        }
        if !self.source.source.is_char_boundary(r.end) {
            return Err(crate::EditErrorKind::NotCharBoundary(r.end));
        }
        let tp = self.source.offset_to_text_pos(r.start)?;
        Ok(Position::new(tp.line(), tp.col(), r, self.file_id))
    }

    pub fn get_span(&self, p: &Position) -> &str {
        self.try_get_span(p)
            .expect("source position must contain a valid source span")
    }

    /// Return a source span if its byte range is valid UTF-8 and lies within
    /// this source file.
    pub fn try_get_span(&self, p: &Position) -> Option<&str> {
        let p_range = p.range();
        // If the span is zero in length then return the single char at that position
        let range = if p_range.is_empty() {
            let end = p_range.start.checked_add(1)?;
            p_range.start..end
        } else {
            p_range
        };

        self.source.source.get(range)
    }
    pub fn get_text(&self) -> &TextFile {
        &self.source
    }
    pub fn get_text_mut(&mut self) -> &mut TextFile {
        Arc::make_mut(&mut self.source)
    }
}

impl Debug for SourceFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut x = f.debug_struct("SourceFile");
        x.field("file", &self.file.to_string_lossy());
        x.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::SourceFile;
    use crate::{AsmSource, Position};

    #[test]
    fn unicode_spans_use_byte_offsets_safely() {
        let source = SourceFile::new("unicode.asm", "a🛰b", AsmSource::FileId(1));
        let position = source.try_get_position(1..5).unwrap();

        assert_eq!(source.try_get_span(&position), Some("🛰"));
        assert!(source.try_get_position(2..3).is_err());
    }

    #[test]
    fn invalid_spans_are_rejected() {
        let source = SourceFile::new("main.asm", "abc", AsmSource::FileId(1));
        let position = Position::new(0, 0, 2..20, AsmSource::FileId(1));

        assert_eq!(source.try_get_span(&position), None);
    }
}
