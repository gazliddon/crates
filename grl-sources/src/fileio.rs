#![deny(unused_imports)]
use grl_utils::{PathSearcher, SearchError, fileutils};
use std::path::{Path, PathBuf};
use anyhow::{anyhow, Result};

pub trait FileIo: PathSearcher {
    fn mk_error(&self, e: SearchError) -> anyhow::Error {
        use SearchError::*;

        match e {
            FileNotFound(f, v) => {
                let errs: Vec<String> = v
                    .into_iter()
                    .map(|f| f.to_string_lossy().to_string())
                    .collect();
                anyhow!(
                    "Can't load {}\n Tried:\n{}",
                    f.to_string_lossy(),
                    errs.join("\n")
                )
            }
            _ => {
                panic!()
            }
        }
    }

    fn add_to_files_read(&mut self, p: PathBuf);
    fn add_to_files_written(&mut self, p: PathBuf);
    fn get_files_written(&self) -> Vec<PathBuf>;
    fn get_files_read(&self) -> Vec<PathBuf>;

    fn read_to_string<P: AsRef<Path>>(&mut self, path: P) -> Result<(PathBuf, String)> {
        let (path,bin) = self.read_binary(path)?;
        let ret = String::from_utf8(bin).unwrap();
        Ok((path, ret))
    }

    fn read_binary<P: AsRef<Path>>(&mut self, path: P) -> Result<(PathBuf, Vec<u8>)> {
        use std::fs::File;
        use std::io::Read;
        let mut buffer = vec![];

        let path = self
            .get_full_path(path)
            .map_err(|e| self.mk_error(e))?;

        let mut file = File::open(path.clone())?;
        file.read_to_end(&mut buffer)?;
        self.add_to_files_read(path.clone());
        Ok((path, buffer))
    }

    fn write<P: AsRef<Path>, C: AsRef<[u8]>>(&mut self, path: P, data: C) -> PathBuf {
        let path = path.as_ref();

        std::fs::write(&path, data).unwrap_or_else(|_| 
            panic!(
             "Can't write bin file {}", path.to_string_lossy() 
                ));

        let abs_path = fileutils::abs_path_from_cwd(&path);

        self.add_to_files_written(abs_path);

        path.to_path_buf()
    }

    fn read_binary_chunk<P: AsRef<Path>>(
        &mut self,
        path: P,
        r: std::ops::Range<usize>,
    ) -> Result<(PathBuf, Vec<u8>)> {
        let (path, buff) = self.read_binary(path)?;

        let buff_r = 0..buff.len();

        let start = r.start;
        let last = (r.len() + r.start) - 1;

        if buff_r.contains(&start) && buff_r.contains(&last) {
            Ok((path, buff[r].into()))
        } else {
            Err(anyhow!(
                "Cannot read binary chunk. Range exceeds size of file {}: file size is {}, tried to grab up to {}",
                path.to_string_lossy(), buff_r.len(), last
                ))
        }
    }

    fn get_size<P: AsRef<Path>>(&self, path: P) -> Result<usize> {
        let path = self
            .get_full_path(path)
            .map_err(|e| self.mk_error(e))?;

        let md = std::fs::metadata(path.clone())
            .map_err(|e| anyhow!("Can't get size for : {}\n{}", path.to_string_lossy(), e))?;

        Ok(md.len() as usize)
    }
}
