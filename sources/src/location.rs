#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Location {
    pub file: String,
    pub line: usize,
}

pub struct SourceChunk {
    pub loc: Location,
    pub lines: usize,
}

impl SourceChunk {}

impl Location {
    pub fn to_source_chunk(&self, lines: usize) -> SourceChunk {
        SourceChunk {
            lines,
            loc: self.clone(),
        }
    }

    pub fn new(file: &str, line: usize) -> Self {
        Self {
            file: file.to_string(),
            line,
        }
    }

    pub fn set_line_number(&mut self, line: usize) {
        self.line = line;
    }

    pub fn get_line_number(&self) -> usize {
        self.line
    }

    pub fn inc_line_number(&mut self) -> usize {
        self.line += 1;
        self.get_line_number()
    }
}
