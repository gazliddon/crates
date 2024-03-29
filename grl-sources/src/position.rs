#![deny(unused_imports)]

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}:{})", self.line, self.col)
    }
}
/// AsmSource is a unique identifier a chunk of sourcode
#[derive(Default,Clone, PartialEq, Eq, Debug, Copy, Hash)]
pub enum AsmSource {
   #[default] 
    FromStr,
    FileId(u64),
}
/// Position is a location in a source file
/// It incorporates a range of characters and is badly named
/// TODO Rename this to TextSpanlLocation
#[derive(Debug, PartialEq, Eq, Clone, Hash, Copy)]
pub struct Position {
    src: AsmSource,
    line: u32,
    col: u32,
    offset : u32,
    len: u32,
}

pub type TextSpanLocation = Position;

impl Position {
    pub fn src(&self) -> AsmSource {
        self.src
    }

    pub fn line_col_from_one(&self) -> (usize, usize) {
        let (l,c) = (self.line + 1, self.col + 1);
        (l as usize, c as usize)
    }

    pub fn line_col(&self) -> (usize, usize) {
        (self.line as usize, self.col as usize)
    }

    pub fn new(line: usize, col: usize, range: std::ops::Range<usize>, src: AsmSource) -> Self {
        Self {
            line: line as u32,
            col: col as u32,
            offset: range.start as u32,
            len: range.len() as u32,
            src,
        }
    }

    pub fn line(&self) -> usize {
        self.line as usize
    }

    pub fn col(&self) -> usize {
        self.col as usize
    }

    pub fn range(&self) -> std::ops::Range<usize> {
        self.offset as usize .. ( self.offset+self.len ) as usize
    }

    /// Does this position overlap with another position
    pub fn overlaps(&self, p: &Position) -> bool {
        let range = self.range();
        let p_range = p.range();
        if self.src == p.src {
            range.end >= p_range.start && range.start < p_range.end
        } else {
            false
        }
    }
    pub fn offset(&self) -> usize {
        self.offset as usize
    }
}

impl Default for Position {
    fn default() -> Self {
        Self {
            line: 0,
            col: 0,
            offset: 0,
            len: 0,
            src: AsmSource::FromStr,
        }
    }
}
