use crate::fileloader::SourceFileLoader;
use crate::FileIo;

use super::{error::*, AsmSource, Position, SourceFile, SourceFiles, TextEditTrait};

use path_clean::PathClean;

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use grl_symbols::SymbolTree;
use grl_utils::fileutils;

pub trait LocationTrait: Clone {
    fn get_line_number(&self) -> usize;
    fn get_file(&self) -> &PathBuf;
}

#[derive(Clone, Debug)]
pub struct SourceLine<'a> {
    pub text: String,
    pub file: PathBuf,
    pub file_id: u64,
    pub line_number: usize,
    pub mapping: Option<&'a Mapping>,
}

impl<'a> LocationTrait for SourceLine<'a> {
    fn get_line_number(&self) -> usize {
        self.line_number
    }

    fn get_file(&self) -> &PathBuf {
        &self.file
    }
}

pub struct SourceFileAccess<'a> {
    source_database: &'a SourceDatabase,
    pub file_id: u64,
    num_of_lines: usize,
}

impl<'a> SourceFileAccess<'a> {
    pub fn new(source_database: &'a SourceDatabase, file_id: u64, num_of_lines: usize) -> Self {
        Self {
            source_database,
            file_id,
            num_of_lines,
        }
    }

    pub fn num_of_lines(&self) -> usize {
        self.num_of_lines
    }

    pub fn get_line(&self, line: usize) -> Option<SourceLine<'a>> {
        self.source_database.get_source_line(self.file_id, line)
    }
}

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Eq, Serialize, Deserialize, Clone)]
pub enum ItemType {
    OpCode,
    Command,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mapping {
    pub file_id: u64,
    pub line: usize,
    pub mem_range: std::ops::Range<usize>,
    pub physical_mem_range: std::ops::Range<usize>,
    pub item_type: ItemType,
}

impl Mapping {
    pub fn contains(&self, addr: usize) -> bool {
        self.physical_mem_range.contains(&addr)
    }
}

use grl_utils::Stack;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SourceMapping {
    pub addr_to_mapping: Vec<Mapping>,
    pub phys_addr_to_mapping: Vec<Mapping>,
    #[serde(skip)]
    macro_stack: Stack<Position>,
}

impl SourceMapping {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_macro(&mut self, pos: &Position) {
        self.macro_stack.push(*pos)
    }

    pub fn stop_macro(&mut self) {
        self.macro_stack.pop();
    }

    fn get_macro_pos(&self) -> Option<&Position> {
        self.macro_stack.front()
    }

    fn is_expanding_macro(&self) -> bool {
        !self.macro_stack.is_empty()
    }

    pub fn get_mapping(&self, addr: usize) -> Option<&Mapping> {
        self.phys_addr_to_mapping.get(addr)
    }

    pub fn add_mapping(
        &mut self,
        physical_mem_range: std::ops::Range<usize>,
        mem_range: std::ops::Range<usize>,
        pos: &Position,
        item_type: ItemType,
    ) {
        let pos = self.get_macro_pos().unwrap_or(pos);

        if let AsmSource::FileId(file_id) = pos.src() {
            let entry = Mapping {
                file_id,
                line: pos.line(),
                mem_range,
                item_type,
                physical_mem_range,
            };

            self.addr_to_mapping.push(entry.clone());
            self.phys_addr_to_mapping.push(entry);
        } else {
            panic!("No file id!")
        }
    }
}
use std::cell::RefCell;

/// Record of a written binary chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinWriteDesc {
    pub file: PathBuf,
    pub addr: std::ops::Range<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinToWrite {
    pub bin_desc: BinWriteDesc,
    pub data: Vec<u8>,
}

impl BinToWrite {
    pub fn new<P: AsRef<Path>>(data: Vec<u8>, p: P, addr: std::ops::Range<usize>) -> Self {
        Self {
            data,
            bin_desc: BinWriteDesc {
                file: p.as_ref().to_path_buf(),
                addr,
            },
        }
    }

    pub fn write_bin(&self, loader: &mut SourceFileLoader) -> (usize, usize, PathBuf) {
        let physical_address = self.bin_desc.addr.start;
        let count = self.bin_desc.addr.len();
        let p = &self.bin_desc.file;
        loader.write(p, &self.data);
        (physical_address, count, self.bin_desc.file.clone())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SourceDatabase {
    id_to_source_file: HashMap<u64, PathBuf>,
    mappings: SourceMapping,

    pub bin_written: Vec<BinWriteDesc>,
    pub exec_addr: Option<usize>,
    pub file_name: PathBuf,

    #[serde(skip)]
    range_to_mapping: HashMap<std::ops::Range<usize>, Mapping>,
    #[serde(skip)]
    phys_addr_to_mapping: HashMap<usize, Mapping>,
    #[serde(skip)]
    addr_to_mapping: HashMap<usize, Mapping>,
    #[serde(skip)]
    source_files: RefCell<HashMap<u64, SourceFile>>,
    #[serde(skip)]
    source_file_to_id: HashMap<PathBuf, u64>,
    #[serde(skip)]
    loc_to_mapping: HashMap<(u64, usize), Mapping>,
}

impl Default for SourceDatabase {
    fn default() -> Self {
        Self {
            source_files: Default::default(),
            source_file_to_id: Default::default(),
            mappings: Default::default(),
            id_to_source_file: Default::default(),
            range_to_mapping: Default::default(),
            addr_to_mapping: Default::default(),
            phys_addr_to_mapping: Default::default(),
            loc_to_mapping: Default::default(),
            bin_written: vec![],
            exec_addr: None,
            file_name: PathBuf::new(),
        }
    }
}

pub struct SourceLineInfo<'a> {
    file: &'a String,
    line: usize,
    mem_range: std::ops::Range<u64>,
}

fn abs_path<P1: AsRef<Path>, P2: AsRef<Path>>(path: P1, base: P2) -> PathBuf {
    let path = path.as_ref();
    let base = base.as_ref().to_path_buf();

    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
    .clean()
}

fn rel_path<P1: AsRef<Path>, P2: AsRef<Path>>(path: P1, base: P2) -> Option<PathBuf> {
    pathdiff::diff_paths(&path, &base)
}

impl SourceDatabase {
    pub fn write_json<P: AsRef<Path>>(&self, file: P) -> std::io::Result<String> {
        let mut copy: SourceDatabase = self.clone();
        copy.file_name = fileutils::abs_path_from_cwd(&file);
        let j = serde_json::to_string_pretty(&copy).expect("Unable to serialize to json");
        fs::write(file, j)?;
        Ok(copy.file_name.to_string_lossy().to_string())
    }

    pub fn new(
        mappings: &SourceMapping,
        sources: &SourceFiles,
        symbols: &SymbolTree<u64, u64, i64>,
        written: &[BinWriteDesc],
        exec_addr: Option<usize>,
    ) -> Self {
        let mut id_to_source_file = HashMap::with_capacity(4096);

        for (k, v) in &sources.id_to_source_file {
            id_to_source_file.insert(*k, v.file.clone());
        }

        let mut ret = Self {
            id_to_source_file,
            exec_addr,
            mappings: mappings.clone(),
            // symbols: symbols.clone(),
            bin_written: written.to_vec(),

            ..Default::default()
        };

        ret.post_deserialize();
        ret
    }

    pub fn from_json<P: AsRef<Path>>(sym_file: P) -> Result<Self, SourceErrorType> {
        use std::io::ErrorKind;
        let file_name = sym_file.as_ref().to_string_lossy();

        println!("Trying to load {:?}", sym_file.as_ref().to_string_lossy());

        let symstr = std::fs::read_to_string(&sym_file).map_err(|e| match e.kind() {
            ErrorKind::NotFound => SourceErrorType::FileNotFound(file_name.to_string()),
            _ => SourceErrorType::Io(e.to_string()),
        })?;

        let mut sd: SourceDatabase =
            serde_json::from_str(&symstr).map_err(|e| SourceErrorType::Io(e.to_string()))?;
        sd.post_deserialize();
        Ok(sd)
    }

    fn post_deserialize(&mut self) {
        let mut file_dir = self.file_name.to_path_buf();
        file_dir.pop();

        for v in &self.mappings.addr_to_mapping {
            self.range_to_mapping.insert(v.mem_range.clone(), v.clone());
            self.addr_to_mapping.insert(v.mem_range.start, v.clone());
            self.phys_addr_to_mapping
                .insert(v.physical_mem_range.start, v.clone());
            let loc = (v.file_id, v.line);
            self.loc_to_mapping.insert(loc, v.clone());
        }

        for (k, v) in &self.id_to_source_file {
            self.source_file_to_id.insert(v.clone(), *k);
        }
    }

    fn load_source_file(&self, file_id: u64) -> Result<(), ()> {
        let file_name = self.id_to_source_file.get(&file_id).ok_or(())?;
        let x = self.source_files.borrow().contains_key(&file_id);

        if !x {
            let s = std::fs::read_to_string(file_name).expect("Should have read source file");
            let mut x = self.source_files.borrow_mut();
            x.insert(file_id, SourceFile::new(file_name, &s, AsmSource::FileId( file_id )));
            x.get(&file_id);
        } else {
            println!("**** Got from cache! {}", file_name.to_string_lossy());
        }

        Ok(())
    }

    pub fn get_source_file_from_file_name<P>(&self, file_name: P) -> Option<SourceFileAccess>
    where
        P: AsRef<Path>,
    {
        self.source_file_to_id
            .get(file_name.as_ref())
            .and_then(|file_id| self.get_source_file(*file_id))
    }

    pub fn get_source_file(&self, file_id: u64) -> Option<SourceFileAccess> {
        self.func_source_file(file_id, |sf| {
            let num_of_lines = sf.source.num_of_lines();
            Some(SourceFileAccess::new(self, file_id, num_of_lines))
        })
    }

    fn get_source_line(&self, file_id: u64, line: usize) -> Option<SourceLine> {
        self.func_source_file(file_id, |sf| {
            sf.get_line(line).map(|text| SourceLine {
                file_id,
                file: sf.file.clone(),
                line_number: line,
                text: text.to_string(),
                mapping: self.loc_to_mapping.get(&(file_id, line)),
            })
        })
    }

    pub fn func_source_file<F, R>(&self, file_id: u64, func: F) -> Option<R>
    where
        F: Fn(&SourceFile) -> Option<R>,
    {
        self.load_source_file(file_id).unwrap();
        self.source_files.borrow().get(&file_id).and_then(func)
    }

    pub fn get_source_line_from_file<P: AsRef<Path>>(
        &self,
        file_name: P,
        line: usize,
    ) -> Option<SourceLine> {
        self.get_source_file_from_file_name(file_name)
            .and_then(|sf| sf.get_line(line))
    }
    pub fn get_source_info_from_physical_address(&self, addr: usize) -> Option<SourceLine> {
        self.phys_addr_to_mapping
            .get(&addr)
            .and_then(|m| self.get_source_line(m.file_id, m.line))
    }

    pub fn get_source_info_from_address(&self, addr: usize) -> Option<SourceLine> {
        self.addr_to_mapping
            .get(&addr)
            .and_then(|m| self.get_source_line(m.file_id, m.line))
    }
}
