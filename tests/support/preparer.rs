use std::{env, error::Error, path::PathBuf, process::Command};

pub(crate) struct Preparer;

impl Preparer {
    pub(crate) fn prepare(scenario: &str) -> Result<PathBuf, Box<dyn Error>> {
        let executable = Self::executable()?;

        let output = Command::new(executable).arg(scenario).output()?;

        if !output.status.success() {
            return Err(format!(
                "Repository preparation failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        let stdout = String::from_utf8(output.stdout)?;
        let root = PathBuf::from(stdout.trim()).canonicalize()?;
        let temporary = env::temp_dir().canonicalize()?;

        if root.parent() != Some(temporary.as_path()) {
            return Err(format!("Repository outside temporary root: {}", root.display()).into());
        }

        if !root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("standardizer-test-"))
        {
            return Err(format!("Unexpected temporary repository: {}", root.display()).into());
        }

        if !root.join(".git").is_dir() {
            return Err(format!("Invalid prepared Git repository: {}", root.display()).into());
        }

        Ok(root)
    }

    fn executable() -> Result<PathBuf, Box<dyn Error>> {
        if let Some(path) = env::var_os("STANDARDIZER_TEST_PREPARER") {
            return Ok(PathBuf::from(path));
        }

        let cargo_home = env::var_os("CARGO_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                env::var_os("HOME")
                    .or_else(|| env::var_os("USERPROFILE"))
                    .map(|home| PathBuf::from(home).join(".cargo"))
            })
            .ok_or("Unable to locate Cargo home")?;

        let executable = cargo_home.join("bin").join(format!(
            "standardizer-prepare-test-repository{}",
            env::consts::EXE_SUFFIX
        ));

        if !executable.is_file() {
            return Err(format!(
                "Missing test preparer: {}. Run scripts/bootstrap.sh first.",
                executable.display()
            )
            .into());
        }

        Ok(executable)
    }
}
