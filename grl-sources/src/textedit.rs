use thiserror::Error;

/// cartesian position in a text file
/// line and character are both zero based
#[derive(Clone, Debug, Copy)]
pub struct TextPos {
    line: usize,
    col: usize,
}

impl TextPos {
    pub fn new(line: usize, character: usize) -> Self {
        Self { line, col: character }
    }
    pub fn col(&self) -> usize {
        self.col
    }
    pub fn line(&self) -> usize {
        self.line
    }
}

/// Contains information for an edit to the in memrory text file
/// start..end is half open, end = the character after the last char to edit
#[derive(Debug)]
pub struct TextEdit<'a> {
    pub start: TextPos,
    pub end: TextPos,
    pub range: std::ops::Range<TextPos>,
    pub text: &'a str,
}

impl<'a> TextEdit<'a> {
    pub fn from_pos(start: TextPos, end: TextPos, text: &'a str) -> Self {
        let range = start.clone()..end.clone();
        Self {
            start,
            end,
            text,
            range,
        }
    }

    pub fn new(
        line_start: usize,
        char_start: usize,
        line_end: usize,
        char_end: usize,
        txt: &'a str,
    ) -> Self {
        let start = TextPos::new(line_start, char_start);
        let end = TextPos::new(line_end, char_end);
        TextEdit::from_pos(start, end, txt)
    }
}

#[derive(Error, Debug, Clone)]
pub enum EditErrorKind {
    #[error("Index out of range, asked for {0}, file size is {1}")]
    IndexOutOfRange(usize, usize),
    #[error("Character out of range: character requesed {0}, line length {1}")]
    CharacterOutOfRange(usize, usize),
    #[error("Line out of range: requested {0}, num of lines {1}")]
    LineOutOfRange(usize, usize),
    #[error("Can't find source file {0}")]
    NoSourceFile(String),
}

pub type EditResult<T> = Result<T, EditErrorKind>;

impl EditErrorKind {
    pub fn char_out_of_range<T>(character: usize, limit: usize) -> EditResult<T> {
        Err(EditErrorKind::CharacterOutOfRange(character, limit))
    }
    pub fn line_out_of_range<T>(line: usize, limit: usize) -> EditResult<T> {
        Err(EditErrorKind::LineOutOfRange(line, limit))
    }
}

pub trait TextEditTrait {
    fn edit(&mut self, _edit: &TextEdit) -> EditResult<()>;
    fn get_line(&self, _line_number: usize) -> EditResult<&str>;
    fn num_of_lines(&self) -> usize;

    fn delete_line(&mut self, line_number: usize) -> EditResult<()> {
        self.replace_line(line_number, "")
    }

    fn replace_line(&mut self, line_number: usize, txt: &str) -> EditResult<()> {
        let text_edit = TextEdit::new(line_number, 0, line_number + 1, 0, txt);
        self.edit(&text_edit)
    }

    fn replace_file(&mut self, new_text: &str) -> EditResult<()>;

    fn insert_line(&mut self, line_number: usize, txt: &str) -> EditResult<()> {
        let txt = &format!("{txt}\n");
        let text_edit = TextEdit::new(line_number, 0, line_number, 0, txt);
        self.edit(&text_edit)
    }

    fn is_empty(&self) -> bool;
}

fn mk_offsets(source: &str) -> Vec<std::ops::Range<usize>> {
    source.lines().map(|x| get_range(source, x)).collect()
}

fn get_range(whole_buffer: &str, part: &str) -> std::ops::Range<usize> {
    let start = part.as_ptr() as usize - whole_buffer.as_ptr() as usize;
    let end = start + part.len();
    start..end
}

#[derive(Clone, PartialEq)]
pub struct TextFile {
    pub source: String,
    pub line_offsets: Vec<std::ops::Range<usize>>,
    pub hash: String,
}

impl std::fmt::Debug for TextFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.source)
    }
}
impl std::fmt::Display for TextFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.source)
    }
}

impl TextEditTrait for TextFile {
    fn is_empty(&self) -> bool {
       self.source.is_empty()
    }

    fn edit(&mut self, edit: &TextEdit) -> EditResult<()> {
        let r = self.get_range(edit)?;
        let last = &self.source[r.end..];
        let new_source = (self.source[..r.start]).to_owned() + edit.text + last;

        self.source = new_source;
        self.post_change();
        Ok(())
    }

    fn replace_file(&mut self, new_text: &str) -> EditResult<()> {
        self.source = new_text.into();
        self.post_change();
        Ok(())
    }

    fn get_line(&self, line_number: usize) -> EditResult<&str> {
        self.line_offsets
            .get(line_number)
            .map(|r| &self.source[r.clone()])
            .ok_or_else(|| EditErrorKind::LineOutOfRange(line_number, self.num_of_lines()))
    }

    fn num_of_lines(&self) -> usize {
        self.line_offsets.len()
    }
}

impl TextFile {
    pub fn new(txt: &str) -> Self {
        let mut ret = Self {
            source: txt.to_string(),
            line_offsets: Default::default(),
            hash: Default::default(),
        };

        ret.post_change();
        ret
    }

    pub fn text_range(&self) -> std::ops::Range<TextPos> {
        let start = TextPos::new(0, 0);
        let end = TextPos::new(self.num_of_lines(), 0);
        start..end
    }

    pub fn get_hash(&self) -> &String {
        &self.hash
    }

    fn post_change(&mut self) {
        self.line_offsets = mk_offsets(&self.source);
        self.rehash();
    }

    fn rehash(&mut self) {
        self.hash = grl_utils::hash::get_hash_from_str(&self.source)
    }

    fn get_line_range(&self, line: usize) -> EditResult<&std::ops::Range<usize>> {
        self.line_offsets
            .get(line)
            .ok_or_else(|| EditErrorKind::LineOutOfRange(line, self.num_of_lines()))
    }

    pub fn start_pos_to_index(&self, pos: &TextPos) -> EditResult<usize> {
        let line_r = self.get_line_range(pos.line())?;
        let ret = line_r.start + pos.col();

        if !line_r.contains(&ret) {
            Err(EditErrorKind::CharacterOutOfRange(pos.col(), line_r.len()))
        } else if ret >= self.source.len() {
            Err(EditErrorKind::IndexOutOfRange(ret, self.source.len()))
        } else {
            Ok(ret)
        }
    }

    fn end_pos_to_index(&self, pos: &TextPos) -> EditResult<usize> {
        if pos.line() == self.num_of_lines() && pos.col() == 0 {
            Ok(self.source.len())
        } else {
            let line_r = self.get_line_range(pos.line)?;

            if pos.col > line_r.len() {
                Err(EditErrorKind::CharacterOutOfRange(pos.col(), line_r.len()))
            } else {
                Ok(line_r.start + pos.col())
            }
        }
    }

    fn get_range(&self, edit: &TextEdit) -> EditResult<std::ops::Range<usize>> {
        let start_index = self.start_pos_to_index(&edit.start)?;
        let end_index = self.end_pos_to_index(&edit.end)?;
        assert!(start_index <= end_index);
        Ok(start_index..end_index)
    }

    /// Take an offset into the file and return a line / character position
    pub fn offset_to_text_pos(&self, offset: usize) -> EditResult<TextPos> {
        let source_len = self.source.len();

        if offset >= source_len {
            Err(EditErrorKind::IndexOutOfRange(offset, source_len))
        } else {
            for (line, l) in self.line_offsets.iter().enumerate() {
                if l.contains(&offset) {
                    let character = offset - l.start;
                    let line = line;
                    return Ok(TextPos { line, col: character });
                }
            }

            panic!("This shouldn't happen")
        }
    }
}

//////////////////////////////////////////////////////////////////////////////////////////

// Implement these traits so I can use TextPos in std::ops::Range
impl PartialEq for TextPos {
    fn eq(&self, other: &Self) -> bool {
        self.line == other.line && self.col == other.col
    }
}

impl PartialOrd for TextPos {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.line.partial_cmp(&other.line) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.col.partial_cmp(&other.col)
    }
}

//////////////////////////////////////////////////////////////////////////////////////////

#[allow(unused_imports)]
mod test {
    // const TEST_TEXT: &str = include_str!("../../assets/test.txt");

    // use super::*;
    // use lazy_static::lazy_static;
    // use pretty_assertions::{assert_eq, assert_ne};

    // #[test]
    // pub fn test_edit() {
    //     let mut text_file = TextFile::new(TEST_TEXT);
    //     assert_eq!(5, text_file.num_of_lines());

    //     let r = edit_test(&mut text_file, 0, 19, 0, 22, "hello");
    //     assert!(r.is_ok());
    //     assert_eq!("Hello this is line hello", text_file.get_line(0).unwrap());

    //     let next_line = text_file.get_line(1).unwrap().to_string();

    //     let r = text_file.delete_line(0);
    //     assert!(r.is_ok());

    //     assert_eq!(&next_line, text_file.get_line(0).unwrap());
    //     assert_eq!(4, text_file.num_of_lines());

    //     let r = edit_test(&mut text_file, 3, 0, 4, 0, "6809 rulez");
    //     assert!(r.is_ok());
    //     assert_eq!("6809 rulez", text_file.get_line(3).unwrap());

    //     let num_of_lines = text_file.num_of_lines();
    //     let r = text_file.insert_line(0, "A new line!");
    //     assert!(r.is_ok());
    //     assert_eq!(text_file.num_of_lines(), num_of_lines + 1);

    //     let r = text_file.delete_line(0);
    //     assert!(r.is_ok());
    //     assert_eq!(text_file.num_of_lines(), num_of_lines);
    // }

    // fn edit_test(
    //     file: &mut TextFile,
    //     line_start: usize,
    //     char_start: usize,
    //     line_end: usize,
    //     char_end: usize,
    //     txt: &str,
    // ) -> EditResult<()> {
    //     let edit = TextEdit::new(line_start, char_start, line_end, char_end, txt);
    //     file.edit(&edit)
    // }
}
