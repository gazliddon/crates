use path_clean::PathClean;
use std::path::{Path, PathBuf};

pub fn rel_path<P1: AsRef<Path>, P2: AsRef<Path>>(path: P1, base: P2) -> Option<PathBuf> {
    pathdiff::diff_paths(&path,&base)
}

pub fn abs_path<P1: AsRef<Path>, P2: AsRef<Path>>(path: P1, base: P2) -> PathBuf {
    let path = path.as_ref();
    let base = base.as_ref().to_path_buf();

    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
    .clean()
}

pub fn abs_path_from_cwd<P1: AsRef<Path>>(path: P1) -> PathBuf {
    let cwd = std::env::current_dir().unwrap();
    abs_path(path,cwd)
}

