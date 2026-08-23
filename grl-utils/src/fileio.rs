#![deny(unused_imports)]
use crate::{fileutils, FResult, FileError, PathSearcher};
use std::path::{Path, PathBuf};

pub trait FileIo: PathSearcher {
    fn add_to_files_read(&mut self, p: PathBuf);
    fn add_to_files_written(&mut self, p: PathBuf);
    fn get_files_written(&self) -> Vec<PathBuf>;
    fn get_files_read(&self) -> Vec<PathBuf>;

    fn read_to_string<P: AsRef<Path>>(&mut self, path: P) -> FResult<(PathBuf, String)> {
        let (path, bin) = self.read_binary(path)?;
        let ret = String::from_utf8(bin).map_err(|error| {
            FileError::Io(format!(
                "source file {} is not UTF-8: {error}",
                path.display()
            ))
        })?;
        Ok((path, ret))
    }

    fn read_binary<P: AsRef<Path>>(&mut self, path: P) -> FResult<(PathBuf, Vec<u8>)> {
        use std::fs::File;
        use std::io::Read;
        let mut buffer = vec![];

        let path = self.get_full_path(path)?;

        let mut file = File::open(path.clone())?;
        file.read_to_end(&mut buffer)?;
        self.add_to_files_read(path.clone());
        Ok((path, buffer))
    }

    fn write<P: AsRef<Path>, C: AsRef<[u8]>>(&mut self, path: P, data: C) -> FResult<PathBuf> {
        let path = path.as_ref();

        std::fs::write(path, data)?;

        let abs_path = fileutils::abs_path_from_cwd(path)?;

        self.add_to_files_written(abs_path);

        Ok(path.to_path_buf())
    }

    fn read_binary_chunk<P: AsRef<Path>>(
        &mut self,
        path: P,
        r: std::ops::Range<usize>,
    ) -> FResult<(PathBuf, Vec<u8>)> {
        let (path, buff) = self.read_binary(path)?;

        if r.start > r.end || r.end > buff.len() {
            let last = r.end.saturating_sub(1);
            return Err(FileError::ReadingBinary(path, buff.len(), last));
        }

        Ok((path, buff[r].into()))
    }

    fn get_size<P: AsRef<Path>>(&self, path: P) -> FResult<usize> {
        let path = self.get_full_path(path)?;

        let md = std::fs::metadata(path.clone()).map_err(|_| FileError::GettingSize(path))?;

        Ok(md.len() as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::FileIo;
    use crate::{PathSearcher, Paths};
    use std::{fs, path::PathBuf};

    #[derive(Default)]
    struct TestFiles {
        paths: Paths,
        read: Vec<PathBuf>,
        written: Vec<PathBuf>,
    }

    impl PathSearcher for TestFiles {
        fn get_full_path<P: AsRef<std::path::Path>>(&self, file: P) -> crate::FResult<PathBuf> {
            self.paths.get_full_path(file)
        }

        fn get_search_paths(&self) -> &[PathBuf] {
            self.paths.get_search_paths()
        }

        fn add_search_path<P: AsRef<std::path::Path>>(&mut self, path: P) {
            self.paths.add_search_path(path)
        }

        fn set_search_paths(&mut self, paths: &[PathBuf]) {
            self.paths.set_search_paths(paths)
        }
    }

    impl FileIo for TestFiles {
        fn add_to_files_read(&mut self, path: PathBuf) {
            self.read.push(path);
        }

        fn add_to_files_written(&mut self, path: PathBuf) {
            self.written.push(path);
        }

        fn get_files_written(&self) -> Vec<PathBuf> {
            self.written.clone()
        }

        fn get_files_read(&self) -> Vec<PathBuf> {
            self.read.clone()
        }
    }

    #[test]
    fn rejects_invalid_binary_ranges_without_panicking() {
        let directory =
            std::env::temp_dir().join(format!("grl-utils-fileio-{}", std::process::id()));
        let file = directory.join("input.bin");
        fs::create_dir_all(&directory).unwrap();
        fs::write(&file, [1u8, 2, 3, 4]).unwrap();

        let mut files = TestFiles::default();
        assert!(files.read_binary_chunk(&file, 1..3).is_ok());
        assert!(files.read_binary_chunk(&file, 3..2).is_err());
        assert!(files.read_binary_chunk(&file, 0..5).is_err());
        assert!(files.read_binary_chunk(&file, 4..4).is_ok());

        fs::remove_dir_all(directory).unwrap();
    }
}
