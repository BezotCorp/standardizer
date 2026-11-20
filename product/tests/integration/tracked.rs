#[test]
fn fix_formats_and_stages_supported_files() {
    let path = prepare("tracked");
    let _cleanup = Cleanup(path.clone());

    let original_rust = read(&path, "example.rs");
    let original_json = read(&path, "example.json");
    let original_txt = read(&path, "example.txt");

    let output = run_standardizer(&path, "fix");

    assert!(
        output.status.success(),
        "fix failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert_ne!(read(&path, "example.rs"), original_rust);
    let biome_executable = path.join("node_modules/.bin/biome");

    let executable = if biome_executable.is_file() {
        biome_executable.into_os_string()
    } else {
        std::ffi::OsString::from("biome")
    };

    let biome = std::process::Command::new(executable)
        .current_dir(&path)
        .arg("--version")
        .output();

    match biome {
        Ok(output) if output.status.success() => {
            assert_ne!(
                read(&path, "example.json"),
                original_json,
                "Biome disponible : le JSON doit être formaté"
            );
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            assert_eq!(
                read(&path, "example.json"),
                original_json,
                "Biome absent : le JSON doit rester inchangé"
            );
        }
        Ok(output) => {
            panic!("Biome présent mais défaillant : {}", output.status);
        }
        Err(error) => {
            panic!("Erreur de détection de Biome : {error}");
        }
    }
    assert_eq!(read(&path, "example.txt"), original_txt);

    for (name, contents) in repository_files(&path) {
        assert_eq!(
            git_index(&path, &name),
            contents,
            "incorrect Git index: {name}"
        );
    }

    assert!(run_standardizer(&path, "check").status.success());
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
fn repository_files(path: &std::path::Path) -> std::collections::BTreeMap<String, Vec<u8>> {
    let mut result = std::collections::BTreeMap::new();

    for entry in std::fs::read_dir(path).expect("read repository") {
        let entry = entry.expect("directory entry");
        let file = entry.path();

        if file.file_name().is_some_and(|name| name == ".git") {
            continue;
        }

        assert!(file.is_file(), "unexpected fixture directory");

        let name = file
            .file_name()
            .expect("filename")
            .to_string_lossy()
            .into_owned();

        result.insert(name, std::fs::read(file).expect("read file"));
    }

    result
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
