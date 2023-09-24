use super::SourceFile;
use super::Position;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SourceInfo<'a> {
    pub fragment: &'a str,
    pub line_str: &'a str,
    pub source_file: &'a SourceFile,
    pub file: PathBuf,
    pub pos: Position,
}
