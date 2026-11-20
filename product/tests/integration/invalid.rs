#[test]
fn invalid_rust_reports_failure() {
    let path = prepare("invalid");
    let _cleanup = Cleanup(path.clone());

    let before = read(&path, "example.rs");
    let index = git_index(&path, "example.rs");

    let output = run_standardizer(&path, "fix");

    assert!(!output.status.success());
    assert_eq!(read(&path, "example.rs"), before);
    assert_eq!(git_index(&path, "example.rs"), index);
}

fn prepare(scenario: &str) -> std::path::PathBuf {
    let executable = std::env::var_os("STANDARDIZER_TEST_PREPARER")
        .expect("STANDARDIZER_TEST_PREPARER is required");

    let output = std::process::Command::new(executable)
        .arg(scenario)
        .output()
        .expect("execute preparer");

    assert!(
        output.status.success(),
        "preparation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("UTF-8 repository path");

    let path = std::path::Path::new(stdout.trim())
        .canonicalize()
        .expect("prepared repository exists");

    let temporary = std::env::temp_dir()
        .canonicalize()
        .expect("temporary root exists");

    assert_eq!(
        path.parent(),
        Some(temporary.as_path()),
        "repository outside temporary root"
    );

    assert!(
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("standardizer-test-")),
        "unexpected temporary repository"
    );

    assert!(path.join(".git").is_dir(), "not a Git repository");

    path
}

struct Cleanup(std::path::PathBuf);

impl Drop for Cleanup {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.0) {
            eprintln!("cleanup failed for {}: {error}", self.0.display());
        }
    }
}

fn run_standardizer(path: &std::path::Path, action: &str) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_standardizer"))
        .current_dir(path)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .arg(action)
        .output()
        .expect("execute standardizer")
}
fn read(path: &std::path::Path, name: &str) -> Vec<u8> {
    std::fs::read(path.join(name)).expect("read file")
}
fn git_index(path: &std::path::Path, name: &str) -> Vec<u8> {
    git(path, &["show", &format!(":{name}")]).stdout
}
fn git(path: &std::path::Path, arguments: &[&str]) -> std::process::Output {
    let output = std::process::Command::new("git")
        .current_dir(path)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .args(arguments)
        .output()
        .expect("execute git");

    assert!(
        output.status.success(),
        "Git failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    output
}
