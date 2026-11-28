use std::{
    error::Error,
    ffi::OsString,
    fs::{self, OpenOptions},
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    process::Command,
};

pub(crate) struct DeveloperConfiguration;

impl DeveloperConfiguration {
    pub(in crate::configuration) const PATH: &str = ".standardizer/dev.json";

    const EXCLUDE_PATTERN: &str = "/.standardizer/dev.json";

    pub(crate) fn protect(root: &Path) -> Result<(), Box<dyn Error>> {
        Self::ensure_excluded(root)?;
        Self::ensure_untracked(root)?;

        Ok(())
    }

    fn git(root: &Path) -> Command {
        let mut command = Command::new("git");

        command.current_dir(root);
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

    fn exclude_path(root: &Path) -> Result<PathBuf, Box<dyn Error>> {
        let mut bytes = Self::output(Self::git(root).args([
            "rev-parse",
            "--path-format=absolute",
            "--git-path",
            "info/exclude",
        ]))?;

        if bytes.last() == Some(&b'\n') {
            bytes.pop();
        }

        Ok(PathBuf::from(Self::os_string(bytes)?))
    }

    fn ensure_excluded(root: &Path) -> Result<(), Box<dyn Error>> {
        let path = Self::exclude_path(root)?;

        let content = match fs::read(&path) {
            Ok(content) => content,
            Err(error) if error.kind() == ErrorKind::NotFound => Vec::new(),
            Err(error) => return Err(error.into()),
        };

        let pattern = Self::EXCLUDE_PATTERN.as_bytes();

        if content
            .split(|byte| *byte == b'\n')
            .any(|line| line.strip_suffix(b"\r").unwrap_or(line) == pattern)
        {
            return Ok(());
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut addition = Vec::new();

        if !content.is_empty() && !content.ends_with(b"\n") {
            addition.push(b'\n');
        }

        addition.extend_from_slice(pattern);
        addition.push(b'\n');

        let mut file = OpenOptions::new().create(true).append(true).open(path)?;

        file.write_all(&addition)?;

        Ok(())
    }

    fn ensure_untracked(root: &Path) -> Result<(), Box<dyn Error>> {
        let entries = Self::output(Self::git(root).args(["ls-files", "-z", "--", Self::PATH]))?;

        if entries.is_empty() {
            return Ok(());
        }

        Self::output(Self::git(root).args(["rm", "--cached", "--force", "--", Self::PATH]))?;

        eprintln!("standardizer: removed {} from Git tracking", Self::PATH);

        Ok(())
    }
}
