
impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}:{})", self.line, self.col)
    }
}

#[derive(Default,Clone, PartialEq, Eq, Debug, Copy, Hash)]
pub enum AsmSource {
   #[default] 
    FromStr,
    FileId(u64),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Position {
    pub line: usize,
    pub col: usize,
    pub range: std::ops::Range<usize>,
    pub src: AsmSource,
}

impl Position {
    pub fn line_col_from_one(&self) -> (usize, usize) {
        (self.line + 1, self.col + 1)
    }

    pub fn line_col(&self) -> (usize, usize) {
        (self.line, self.col)
    }

    pub fn new(line: usize, col: usize, range: std::ops::Range<usize>, src: AsmSource) -> Self {
        Self {
            line,
            col,
            range,
            src,
        }
    }

    pub fn overlaps(&self, p: &Position) -> bool {
        if self.src == p.src {
            self.range.end >= p.range.start && self.range.start < p.range.end
        } else {
            false
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self {
            line: 0,
            col: 0,
            range: 0..0,
            src: AsmSource::FromStr,
        }
    }
}
