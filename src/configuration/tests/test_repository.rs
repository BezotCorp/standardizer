use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

pub(in crate::configuration::tests) struct TestRepository {
    root: PathBuf,
}

impl TestRepository {
    pub(in crate::configuration::tests) fn new() -> Result<Self, Box<dyn Error>> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();

        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);

        let root = std::env::temp_dir().join(format!(
            "standardizer-configuration-{}-{timestamp}-{id}",
            std::process::id()
        ));

        fs::create_dir(&root)?;

        let repository = Self { root };

        repository.git(&["init", "--quiet"])?;
        repository.git(&["config", "user.name", "Standardizer Test"])?;
        repository.git(&["config", "user.email", "test@example.invalid"])?;
        repository.git(&["config", "commit.gpgsign", "false"])?;

        fs::create_dir_all(repository.root.join(".git/test-hooks"))?;

        repository.git(&["config", "core.hooksPath", ".git/test-hooks"])?;

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
        let output = Command::new("git")
            .current_dir(&self.root)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_INDEX_FILE")
            .env_remove("GIT_COMMON_DIR")
            .env_remove("GIT_OBJECT_DIRECTORY")
            .env_remove("GIT_ALTERNATE_OBJECT_DIRECTORIES")
            .args(arguments)
            .output()?;

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
