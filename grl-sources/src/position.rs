
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

#[derive(Debug, PartialEq, Eq, Clone, Hash, Copy)]
pub struct Position {
    pub line: usize,
    pub col: usize,
    pub offset : usize,
    pub len: usize,
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
            offset: range.start,
            len: range.len(),
            src,
        }
    }

    pub fn line(&self) -> usize {
        self.line
    }
    pub fn col(&self) -> usize {
        self.col
    }

    pub fn range(&self) -> std::ops::Range<usize> {
        self.offset .. self.offset+self.len
    }

    pub fn overlaps(&self, p: &Position) -> bool {
        let range = self.range();
        let p_range = p.range();
        if self.src == p.src {
            range.end >= p_range.start && range.start < p_range.end
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
            offset: 0,
            len: 0,
            src: AsmSource::FromStr,
        }
    }
}
