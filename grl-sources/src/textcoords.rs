/// cartesian position in a text file
/// line and character are both zero based
#[derive(Clone, Debug, Copy)]
pub struct TextCoords {
    line: u32,
    col: u32,
}

impl TextCoords {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line: line as u32, col: col as u32 }
    }
    pub fn col(&self) -> usize {
        self.col as usize
    }
    pub fn line(&self) -> usize {
        self.line as usize
    }
}

// Implement these traits so I can use TextPos in std::ops::Range
impl PartialEq for TextCoords {
    fn eq(&self, other: &Self) -> bool {
        self.line == other.line && self.col == other.col
    }
}

impl PartialOrd for TextCoords {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.line.partial_cmp(&other.line) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.col.partial_cmp(&other.col)
    }
}
