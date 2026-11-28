use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub(in crate::configuration::tests) struct TestRepository {
    root: PathBuf,
}

impl TestRepository {
    pub(in crate::configuration::tests) fn new() -> Result<Self, Box<dyn Error>> {
        let root = crate::configuration::tests::preparer::Preparer::prepare("untracked")?;

        let repository = Self { root };

        repository.git(&["clean", "-fd"])?;

        Ok(repository)
    }

    pub(in crate::configuration::tests) fn root(&self) -> &Path {
        &self.root
    }

    pub(in crate::configuration::tests) fn write(
        &self,
        path: &str,
        content: &str,
    ) -> Result<(), Box<dyn Error>> {
        let path = self.root.join(path);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, content)?;

        Ok(())
    }

    pub(in crate::configuration::tests) fn git(
        &self,
        arguments: &[&str],
    ) -> Result<String, Box<dyn Error>> {
        let mut command = Command::new("git");

        command.current_dir(&self.root);

        for (key, _) in std::env::vars_os() {
            if key.to_string_lossy().starts_with("GIT_") {
                command.env_remove(key);
            }
        }

        command.env("GIT_CONFIG_NOSYSTEM", "1");
        command.env(
            "GIT_CONFIG_GLOBAL",
            self.root.join(".git/test-global-config"),
        );
        command.env("LC_ALL", "C");

        let output = command.args(arguments).output()?;

        if !output.status.success() {
            return Err(format!(
                "git {arguments:?}: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        Ok(String::from_utf8(output.stdout)?)
    }
}

impl Drop for TestRepository {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
