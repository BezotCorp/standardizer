use crate::{goals::ConformanceIntent, tracked_file::TrackedFile};
use std::{
    collections::BTreeMap,
    error::Error,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub(crate) struct GitRepository {
    root: PathBuf,
}

impl GitRepository {
    pub(crate) fn discover() -> Result<Self, Box<dyn Error>> {
        let output = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()?;

        if !output.status.success() {
            return Err("cannot find Git repository".into());
        }

        let mut bytes = output.stdout;
        if bytes.last() == Some(&b'\n') {
            bytes.pop();
        }

        Ok(Self {
            root: PathBuf::from(Self::os_string(bytes)?),
        })
    }

    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    fn git(&self) -> Command {
        let mut command = Command::new("git");
        command.current_dir(&self.root);
        command.env("GIT_LITERAL_PATHSPECS", "1");
        command
    }

    fn output(command: &mut Command) -> Result<Vec<u8>, Box<dyn Error>> {
        let output = command.output()?;
        if !output.status.success() {
            return Err(format!(
                "command failed: {command:?}: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }
        Ok(output.stdout)
    }

    #[cfg(unix)]
    fn os_string(bytes: Vec<u8>) -> Result<OsString, Box<dyn Error>> {
        use std::os::unix::ffi::OsStringExt;
        Ok(OsString::from_vec(bytes))
    }

    #[cfg(not(unix))]
    fn os_string(bytes: Vec<u8>) -> Result<OsString, Box<dyn Error>> {
        Ok(OsString::from(String::from_utf8(bytes)?))
    }

    fn hash(&self, path: &Path) -> Result<OsString, Box<dyn Error>> {
        let mut bytes = Self::output(self.git().arg("hash-object").arg("--").arg(path))?;

        if bytes.last() == Some(&b'\n') {
            bytes.pop();
        }

        Self::os_string(bytes)
    }

    pub(crate) fn candidates(&self) -> Result<Vec<TrackedFile>, Box<dyn Error>> {
        let entries = Self::output(self.git().args(["ls-files", "--stage", "-z"]))?;

        let mut files = Vec::new();

        for record in entries
            .split(|byte| *byte == 0)
            .filter(|record| !record.is_empty())
        {
            let tab = record
                .iter()
                .position(|byte| *byte == b'\t')
                .ok_or("invalid Git index entry")?;

            let metadata = std::str::from_utf8(&record[..tab])?;
            let mut fields = metadata.split(' ');

            let mode = fields.next().ok_or("missing Git mode")?;
            let blob = OsString::from(fields.next().ok_or("missing Git blob")?);
            let stage = fields.next().ok_or("missing Git stage")?;

            if stage != "0" {
                return Err("unresolved Git conflicts".into());
            }

            if !matches!(mode, "100644" | "100755") {
                continue;
            }

            let path = PathBuf::from(Self::os_string(record[tab + 1..].to_vec())?);

            let Ok(metadata) = fs::symlink_metadata(self.root.join(&path)) else {
                continue;
            };

            if !metadata.is_file() {
                continue;
            }

            if self.hash(&path)? != blob {
                continue;
            }

            let file = TrackedFile::new(path, blob, mode.to_owned());

            if self.matches(&file, file.blob())? {
                files.push(file);
            }
        }

        Ok(files)
    }

    fn index_matches(&self, file: &TrackedFile) -> Result<bool, Box<dyn Error>> {
        let entries = Self::output(
            self.git()
                .args(["ls-files", "--stage", "-z", "--"])
                .arg(file.path()),
        )?;

        let mut records = entries
            .split(|byte| *byte == 0)
            .filter(|record| !record.is_empty());

        let Some(record) = records.next() else {
            return Ok(false);
        };

        if records.next().is_some() {
            return Ok(false);
        }

        let Some(tab) = record.iter().position(|byte| *byte == b'\t') else {
            return Err("invalid Git index entry".into());
        };

        let metadata = std::str::from_utf8(&record[..tab])?;
        let mut fields = metadata.split(' ');

        let mode = fields.next();
        let blob = fields.next();
        let stage = fields.next();

        Ok(mode == Some(file.mode())
            && blob.map(OsString::from).as_ref() == Some(file.blob())
            && stage == Some("0")
            && Self::os_string(record[tab + 1..].to_vec())? == file.path())
    }

    fn regular_file_matches(&self, file: &TrackedFile) -> Result<bool, Box<dyn Error>> {
        let absolute = self.root.join(file.path());

        let metadata = match fs::symlink_metadata(&absolute) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(false);
            }
            Err(error) => return Err(error.into()),
        };

        if !metadata.is_file() {
            return Ok(false);
        }

        let mut parent = file.path().parent();

        while let Some(path) = parent {
            if path.as_os_str().is_empty() {
                break;
            }

            let metadata = match fs::symlink_metadata(self.root.join(path)) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    return Ok(false);
                }
                Err(error) => return Err(error.into()),
            };

            if !metadata.is_dir() {
                return Ok(false);
            }

            parent = path.parent();
        }

        let changes = Self::output(
            self.git()
                .args(["diff", "--raw", "--no-renames", "--no-ext-diff", "-z", "--"])
                .arg(file.path()),
        )?;

        if changes.is_empty() {
            return Ok(true);
        }

        let header = changes
            .split(|byte| *byte == 0)
            .next()
            .ok_or("invalid Git diff entry")?;

        let header = std::str::from_utf8(header)?;
        let mut fields = header
            .strip_prefix(':')
            .ok_or("invalid Git diff header")?
            .split_whitespace();

        Ok(fields.next() == Some(file.mode()) && fields.next() == Some(file.mode()))
    }

    pub(crate) fn matches(
        &self,
        file: &TrackedFile,
        expected: &OsString,
    ) -> Result<bool, Box<dyn Error>> {
        if !self.index_matches(file)? || !self.regular_file_matches(file)? {
            return Ok(false);
        }

        Ok(self.hash(file.path())? == *expected)
    }

    pub(crate) fn capture(&self, file: &TrackedFile) -> Result<Option<OsString>, Box<dyn Error>> {
        if !self.index_matches(file)? || !self.regular_file_matches(file)? {
            return Ok(None);
        }

        Ok(Some(self.hash(file.path())?))
    }

    fn pending_deletions(&self) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        let output = Self::output(self.git().args([
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
            "--no-renames",
            "--ignore-submodules=none",
        ]))?;

        let mut deletions = Vec::new();

        for record in output.split(|byte| *byte == 0) {
            if record.is_empty() {
                continue;
            }

            if record.len() < 4 || record[2] != b' ' {
                return Err("invalid Git status entry".into());
            }

            let index = record[0];
            let worktree = record[1];

            if index == b'?' || index == b'U' || worktree == b'U' {
                return Ok(Vec::new());
            }

            if matches!((index, worktree), (b'A', b'A') | (b'D', b'D')) {
                return Ok(Vec::new());
            }

            match worktree {
                b' ' => {}
                b'D' => {
                    deletions.push(PathBuf::from(Self::os_string(record[3..].to_vec())?));
                }
                _ => return Ok(Vec::new()),
            }
        }

        Ok(deletions)
    }

    pub(crate) fn stage_deletions(&self) -> Result<(), Box<dyn Error>> {
        let deletions = self.pending_deletions()?;

        if deletions.is_empty() {
            return Ok(());
        }

        if self.pending_deletions()? != deletions {
            return Ok(());
        }

        for path in &deletions {
            match fs::symlink_metadata(self.root.join(path)) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
                Ok(_) => return Ok(()),
            }
        }

        Self::output(
            self.git()
                .args(["rm", "--cached", "--force", "--"])
                .args(&deletions),
        )?;

        for path in deletions {
            eprintln!(
                "standardizer: automatically staged deletion {}",
                path.display(),
            );
        }

        Ok(())
    }

    pub(crate) fn stage_changed(
        &self,
        files: &[TrackedFile],
        expected: &BTreeMap<PathBuf, OsString>,
        intent: ConformanceIntent,
    ) -> Result<(), Box<dyn Error>> {
        if intent != ConformanceIntent::FixConformance {
            return Ok(());
        }

        for file in files {
            let Some(blob) = expected.get(file.path()) else {
                continue;
            };

            if blob == file.blob() {
                continue;
            }

            if !self.matches(file, blob)? {
                eprintln!(
                    "standardizer: skipping changed file {}",
                    file.path().display(),
                );
                continue;
            }

            Self::output(self.git().arg("add").arg("--").arg(file.path()))?;
        }

        Ok(())
    }
}
