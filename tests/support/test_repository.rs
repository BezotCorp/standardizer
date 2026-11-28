use std::{
    env,
    error::Error,
    fs,
    path::PathBuf,
    process::{self, Command},
    sync::OnceLock,
};

pub(crate) struct TestRepository {
    root: PathBuf,
}

impl TestRepository {
    pub(crate) fn new() -> Result<Self, Box<dyn Error>> {
        let repository = Self::from_scenario("untracked")?;

        repository.git(&["clean", "-fd"])?;

        let exclude = repository.root.join(".git/info/exclude");
        let mut exclusions = fs::read_to_string(&exclude)?;

        if !exclusions.ends_with('\n') {
            exclusions.push('\n');
        }

        exclusions.push_str("/.test-tools/\n");
        fs::write(exclude, exclusions)?;

        repository.install_formatter()?;

        Ok(repository)
    }

    fn command(&self, executable: &str) -> Command {
        let mut command = Command::new(executable);
        command.current_dir(&self.root);

        for (key, _) in env::vars_os() {
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

        command
    }

    pub(crate) fn git(&self, arguments: &[&str]) -> Result<String, Box<dyn Error>> {
        let output = self.command("git").args(arguments).output()?;

        if !output.status.success() {
            return Err(format!(
                "git {arguments:?}: {}",
                String::from_utf8_lossy(&output.stderr),
            )
            .into());
        }

        Ok(String::from_utf8(output.stdout)?)
    }

    pub(crate) fn write(&self, path: &str, content: &str) -> Result<(), Box<dyn Error>> {
        let path = self.root.join(path);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, content)?;

        Ok(())
    }

    pub(crate) fn read(&self, path: &str) -> Result<String, Box<dyn Error>> {
        Ok(fs::read_to_string(self.root.join(path))?)
    }

    pub(crate) fn exists(&self, path: &str) -> bool {
        self.root.join(path).exists()
    }

    pub(crate) fn tracked(&self, path: &str, content: &str) -> Result<(), Box<dyn Error>> {
        self.write(path, content)?;
        self.git(&["add", "--", path])?;
        self.git(&["commit", "--quiet", "-m", "chore: add test fixture"])?;

        Ok(())
    }

    pub(crate) fn index(&self, path: &str) -> Result<String, Box<dyn Error>> {
        self.git(&["show", &format!(":{path}")])
    }

    pub(crate) fn status(&self) -> Result<String, Box<dyn Error>> {
        self.git(&["status", "--porcelain=v1", "--untracked-files=all"])
    }

    pub(crate) fn fix(&self, scenario: &str) -> Result<(), Box<dyn Error>> {
        self.run("fix", scenario)
    }

    pub(crate) fn check(&self) -> Result<(), Box<dyn Error>> {
        self.run("check", "normal")
    }

    fn run(&self, intent: &str, scenario: &str) -> Result<(), Box<dyn Error>> {
        let mut paths = vec![self.root.join(".test-tools")];

        if let Some(path) = env::var_os("PATH") {
            paths.extend(env::split_paths(&path));
        }

        let output = self
            .command(env!("CARGO_BIN_EXE_standardizer"))
            .arg(intent)
            .env("PATH", env::join_paths(paths)?)
            .env("STANDARDIZER_TEST_SCENARIO", scenario)
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "standardizer {intent} failed:\n{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            )
            .into());
        }

        Ok(())
    }

    pub(crate) fn from_scenario(scenario: &str) -> Result<Self, Box<dyn Error>> {
        let root = crate::support::Preparer::prepare(scenario)?;

        Ok(Self { root })
    }

    pub(crate) fn run_real(&self, intent: &str) -> Result<(), Box<dyn Error>> {
        let output = self
            .command(env!("CARGO_BIN_EXE_standardizer"))
            .arg(intent)
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "standardizer {intent} failed:\\n{}\\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            )
            .into());
        }

        Ok(())
    }

    fn install_formatter(&self) -> Result<(), Box<dyn Error>> {
        static FORMATTER: OnceLock<Result<PathBuf, String>> = OnceLock::new();

        let executable = FORMATTER
            .get_or_init(|| Self::compile_formatter().map_err(|error| error.to_string()))
            .as_ref()
            .map_err(|error| error.clone())?;

        let directory = self.root.join(".test-tools");

        fs::create_dir_all(&directory)?;

        fs::copy(
            executable,
            directory.join(format!("rustfmt{}", env::consts::EXE_SUFFIX,)),
        )?;

        Ok(())
    }

    fn compile_formatter() -> Result<PathBuf, Box<dyn Error>> {
        let binary = PathBuf::from(env!("CARGO_BIN_EXE_standardizer"));

        let directory = binary
            .parent()
            .ok_or("missing binary directory")?
            .join("standardizer-test-tools");

        fs::create_dir_all(&directory)?;

        let executable = directory.join(format!(
            "formatter-{}{}",
            process::id(),
            env::consts::EXE_SUFFIX,
        ));

        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/support/formatter.rs");

        let compiler = env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());

        let output = Command::new(compiler)
            .arg("--edition=2024")
            .arg("--crate-name")
            .arg("standardizer_test_formatter")
            .arg(source)
            .arg("-o")
            .arg(&executable)
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "formatter compilation failed: {}",
                String::from_utf8_lossy(&output.stderr),
            )
            .into());
        }

        Ok(executable)
    }
}

impl Drop for TestRepository {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
