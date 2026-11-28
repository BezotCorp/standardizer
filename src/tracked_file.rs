use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

pub(crate) struct TrackedFile {
    path: PathBuf,
    blob: OsString,
    mode: String,
}

impl TrackedFile {
    pub(crate) fn new(path: PathBuf, blob: OsString, mode: String) -> Self {
        Self { path, blob, mode }
    }

    pub(crate) fn mode(&self) -> &str {
        &self.mode
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn blob(&self) -> &OsString {
        &self.blob
    }
}
