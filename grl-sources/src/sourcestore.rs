#![deny(unused_imports)]

use super::{
    fileloader::SourceFileLoader, AsmSource, Position, SourceFile, SourceFiles, TextEditTrait,
};

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use grl_utils::FileIo;

#[derive(Clone, Debug)]
pub struct SourceLine<'a> {
    pub text: String,
    pub file: PathBuf,
    pub file_id: u64,
    pub line_number: usize,
    pub mapping: Option<&'a Mapping>,
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

    /// Return a snapshot of the complete source file for syntax highlighting.
    pub fn get_entire_source(&self) -> Option<String> {
        self.source_database
            .func_source_file(self.file_id, |source| {
                Some(source.get_entire_source().to_owned())
            })
    }
}

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ItemType {
    OpCode,
    Command,
    Other,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SourceMapping {
    /// Canonical mapping records. The two public index vectors contain
    /// indices into this list rather than cloned `Mapping` values.
    pub mappings: Vec<Mapping>,
    pub addr_to_mapping: Vec<usize>,
    pub phys_addr_to_mapping: Vec<usize>,
    #[cfg_attr(feature = "serde", serde(skip))]
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

    pub fn get_mapping(&self, addr: usize) -> Option<&Mapping> {
        self.phys_addr_to_mapping
            .iter()
            .filter_map(|index| self.mappings.get(*index))
            .find(|mapping| mapping.physical_mem_range.contains(&addr))
    }

    pub fn add_mapping(
        &mut self,
        physical_mem_range: std::ops::Range<usize>,
        mem_range: std::ops::Range<usize>,
        pos: &Position,
        item_type: ItemType,
    ) {
        let pos = self.get_macro_pos().unwrap_or(pos);

        let AsmSource::FileId(file_id) = pos.src() else {
            // Source generated from an in-memory string has no stable file
            // identity, so it cannot participate in source-map lookups.
            return;
        };

        let entry = Mapping {
            file_id,
            line: pos.line(),
            mem_range,
            item_type,
            physical_mem_range,
        };

        let index = self.mappings.len();
        self.mappings.push(entry);
        self.addr_to_mapping.push(index);
        self.phys_addr_to_mapping.push(index);
    }
}
use std::cell::RefCell;

/// Record of a written binary chunk
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BinWriteDesc {
    pub file: PathBuf,
    pub addr: std::ops::Range<usize>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

    pub fn write_bin(
        &self,
        loader: &mut SourceFileLoader,
    ) -> grl_utils::FResult<(usize, usize, PathBuf)> {
        let physical_address = self.bin_desc.addr.start;
        let count = self.bin_desc.addr.len();
        let p = &self.bin_desc.file;
        loader.write(p, &self.data)?;
        Ok((physical_address, count, self.bin_desc.file.clone()))
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SourceDatabase {
    id_to_source_file: HashMap<u64, PathBuf>,
    mappings: SourceMapping,

    pub bin_written: Vec<BinWriteDesc>,
    pub exec_addr: Option<usize>,
    pub file_name: PathBuf,

    #[cfg_attr(feature = "serde", serde(skip))]
    range_to_mapping: HashMap<std::ops::Range<usize>, usize>,
    #[cfg_attr(feature = "serde", serde(skip))]
    phys_addr_to_mapping: HashMap<usize, usize>,
    #[cfg_attr(feature = "serde", serde(skip))]
    addr_to_mapping: HashMap<usize, usize>,
    #[cfg_attr(feature = "serde", serde(skip))]
    source_files: RefCell<HashMap<u64, SourceFile>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    source_file_to_id: HashMap<PathBuf, u64>,
    #[cfg_attr(feature = "serde", serde(skip))]
    loc_to_mapping: HashMap<(u64, usize), usize>,
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

impl SourceDatabase {
    pub fn new(
        mappings: &SourceMapping,
        sources: &SourceFiles,
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

        ret.rebuild_indexes();
        ret
    }

    /// Rebuild the lookup indexes omitted from Serde output.
    ///
    /// Call this after deserializing a `SourceDatabase`. The derived Serde
    /// implementation intentionally skips caches and reverse indexes because
    /// they are runtime state and can be reconstructed from the persisted
    /// source IDs and mappings.
    pub fn rebuild_indexes(&mut self) {
        self.range_to_mapping.clear();
        self.addr_to_mapping.clear();
        self.phys_addr_to_mapping.clear();
        self.loc_to_mapping.clear();
        self.source_file_to_id.clear();
        for (index, v) in self.mappings.mappings.iter().enumerate() {
            self.range_to_mapping.insert(v.mem_range.clone(), index);
            self.addr_to_mapping.insert(v.mem_range.start, index);
            self.phys_addr_to_mapping
                .insert(v.physical_mem_range.start, index);
            let loc = (v.file_id, v.line);
            self.loc_to_mapping.insert(loc, index);
        }

        for (k, v) in &self.id_to_source_file {
            self.source_file_to_id.insert(v.clone(), *k);
        }
    }

    fn load_source_file(&self, file_id: u64) -> Result<(), ()> {
        let file_name = self.id_to_source_file.get(&file_id).ok_or(())?;
        let x = self.source_files.borrow().contains_key(&file_id);

        if !x {
            let s = std::fs::read_to_string(file_name).map_err(|_| ())?;
            let mut x = self.source_files.borrow_mut();
            x.insert(
                file_id,
                SourceFile::new(file_name, &s, AsmSource::FileId(file_id)),
            );
            x.get(&file_id);
        } else {
            // Source file is already cached.
        }

        Ok(())
    }

    pub fn get_source_file_from_file_name<P>(&self, file_name: P) -> Option<SourceFileAccess<'_>>
    where
        P: AsRef<Path>,
    {
        self.source_file_to_id
            .get(file_name.as_ref())
            .and_then(|file_id| self.get_source_file(*file_id))
    }

    pub fn get_source_file(&self, file_id: u64) -> Option<SourceFileAccess<'_>> {
        self.func_source_file(file_id, |sf| {
            let num_of_lines = sf.get_text().num_of_lines();
            Some(SourceFileAccess::new(self, file_id, num_of_lines))
        })
    }

    fn get_source_line(&self, file_id: u64, line: usize) -> Option<SourceLine<'_>> {
        self.func_source_file(file_id, |sf| {
            sf.get_line(line).map(|text| SourceLine {
                file_id,
                file: sf.file.clone(),
                line_number: line,
                text: text.to_string(),
                mapping: self
                    .loc_to_mapping
                    .get(&(file_id, line))
                    .and_then(|index| self.mappings.mappings.get(*index)),
            })
        })
    }

    pub fn func_source_file<F, R>(&self, file_id: u64, func: F) -> Option<R>
    where
        F: Fn(&SourceFile) -> Option<R>,
    {
        self.load_source_file(file_id).ok()?;
        self.source_files.borrow().get(&file_id).and_then(func)
    }

    pub fn get_source_line_from_file<P: AsRef<Path>>(
        &self,
        file_name: P,
        line: usize,
    ) -> Option<SourceLine<'_>> {
        self.get_source_file_from_file_name(file_name)
            .and_then(|sf| sf.get_line(line))
    }
    pub fn get_source_info_from_physical_address(&self, addr: usize) -> Option<SourceLine<'_>> {
        self.phys_addr_to_mapping
            .get(&addr)
            .and_then(|index| self.mappings.mappings.get(*index))
            .and_then(|m| self.get_source_line(m.file_id, m.line))
    }

    pub fn get_source_info_from_address(&self, addr: usize) -> Option<SourceLine<'_>> {
        self.addr_to_mapping
            .get(&addr)
            .and_then(|index| self.mappings.mappings.get(*index))
            .and_then(|m| self.get_source_line(m.file_id, m.line))
    }
}

#[cfg(test)]
mod tests {
    use super::{ItemType, SourceMapping};
    use crate::{AsmSource, Position};

    #[test]
    fn mapping_lookup_uses_physical_ranges() {
        let mut mappings = SourceMapping::new();
        let position = Position::new(3, 0, 0..4, AsmSource::FileId(7));
        mappings.add_mapping(0x100..0x104, 0x200..0x204, &position, ItemType::OpCode);

        assert!(mappings.get_mapping(0x100).is_some());
        assert!(mappings.get_mapping(0x103).is_some());
        assert!(mappings.get_mapping(0x104).is_none());
    }

    #[test]
    fn mappings_without_file_ids_are_ignored() {
        let mut mappings = SourceMapping::new();
        let position = Position::new(0, 0, 0..1, AsmSource::FromStr);

        mappings.add_mapping(0..1, 0..1, &position, ItemType::OpCode);

        assert!(mappings.addr_to_mapping.is_empty());
        assert!(mappings.phys_addr_to_mapping.is_empty());
    }
}
