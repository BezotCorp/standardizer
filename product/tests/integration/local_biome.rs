use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process::{Command, Output},
};

#[test]
fn locally_installed_biome_formats_and_stages_json() {
    let path = prepare("tracked");
    let _cleanup = Cleanup(path.clone());

    let original_json = fs::read(path.join("example.json")).expect("read original JSON");

    let original_txt = fs::read(path.join("example.txt")).expect("read original text");

    let biome = path.join("node_modules/.bin/biome");

    assert!(
        !biome.exists(),
        "fixture must not contain a local Biome installation"
    );

    fs::write(
        path.join("package.json"),
        r#"{
  "private": true,
  "devDependencies": {
    "@biomejs/biome": "2.6.8"
  }
}
"#,
    )
    .expect("create project package.json");

    install_biome(&path);

    assert!(
        biome.is_file(),
        "npm did not install node_modules/.bin/biome"
    );

    let version = Command::new(&biome)
        .current_dir(&path)
        .arg("--version")
        .output()
        .expect("execute locally installed Biome");

    assert_success("local Biome version", &version);

    let output = standardizer(&path, "fix");

    assert_success("standardizer fix", &output);

    let formatted_json = fs::read(path.join("example.json")).expect("read formatted JSON");

    assert_ne!(
        formatted_json, original_json,
        "Standardizer must format JSON using local Biome"
    );

    assert_eq!(
        formatted_json,
        git(&path, &["show", ":example.json"]).stdout,
        "formatted JSON must be staged"
    );

    assert_eq!(
        fs::read(path.join("example.txt")).expect("read text"),
        original_txt,
        "unsupported files must remain unchanged"
    );

    let check = standardizer(&path, "check");

    assert_success("standardizer check", &check);
}

fn install_biome(path: &Path) {
    let manager = detect_package_manager();

    let mut command = Command::new(manager.name);
    command.current_dir(path);

    match manager.name {
        "pnpm" => {
            let store = path.join(".pnpm-store");

            command.args([
                "install",
                "--ignore-scripts",
                "--no-frozen-lockfile",
                "--store-dir",
            ]);

            command.arg(store);
        }
        "npm" => {
            let cache = path.join(".npm-cache");

            command.args([
                "install",
                "--no-package-lock",
                "--ignore-scripts",
                "--no-audit",
                "--no-fund",
                "--cache",
            ]);

            command.arg(cache);
        }
        "yarn" if manager.major == Some(1) => {
            command.args(["install", "--ignore-scripts", "--no-lockfile"]);

            command.env("YARN_CACHE_FOLDER", path.join(".yarn-cache"));
        }
        "yarn" => {
            command.arg("install");

            command.env("YARN_NODE_LINKER", "node-modules");
            command.env("YARN_ENABLE_SCRIPTS", "false");
            command.env("YARN_ENABLE_GLOBAL_CACHE", "false");
            command.env("YARN_ENABLE_IMMUTABLE_INSTALLS", "false");
            command.env("YARN_CACHE_FOLDER", path.join(".yarn-cache"));
        }
        _ => unreachable!("unsupported package manager"),
    }

    // Éviter toute modification des outils globaux.
    command.env("COREPACK_ENABLE_AUTO_PIN", "0");

    let installation = command.output().expect("execute selected package manager");

    assert_success(
        &format!("local Biome installation using {}", manager.name),
        &installation,
    );
}

struct PackageManager {
    name: &'static str,
    major: Option<u64>,
}

fn detect_package_manager() -> PackageManager {
    for name in ["pnpm", "npm", "yarn"] {
        let result = Command::new(name).arg("--version").output();

        match result {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8(output.stdout)
                    .expect("package manager version must be UTF-8");

                let major = version
                    .trim()
                    .split('.')
                    .next()
                    .and_then(|part| part.parse::<u64>().ok());

                if name == "yarn" && major.is_none() {
                    panic!("Cannot determine Yarn version: {}", version.trim());
                }

                return PackageManager { name, major };
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                // Gestionnaire absent : essayer le suivant.
            }
            Ok(output) => {
                panic!(
                    "{name} is present but --version failed:\n{}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            Err(error) => {
                panic!("Cannot execute {name}: {error}");
            }
        }
    }

    panic!(
        "No JavaScript package manager found. \
         Install pnpm, npm or Yarn to run the local Biome integration test."
    );
}

fn prepare(scenario: &str) -> PathBuf {
    let executable =
        env::var_os("STANDARDIZER_TEST_PREPARER").expect("STANDARDIZER_TEST_PREPARER is required");

    let output = Command::new(executable)
        .arg(scenario)
        .output()
        .expect("execute preparer");

    assert_success("prepare test repository", &output);

    let stdout = String::from_utf8(output.stdout).expect("UTF-8 repository path");

    let path = Path::new(stdout.trim())
        .canonicalize()
        .expect("prepared repository exists");

    let temporary = env::temp_dir()
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

    assert!(
        path.join(".git").is_dir(),
        "prepared directory is not a Git repository"
    );

    path
}

fn standardizer(path: &Path, action: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_standardizer"))
        .current_dir(path)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .arg(action)
        .output()
        .expect("execute Standardizer")
}

fn git(path: &Path, arguments: &[&str]) -> Output {
    let output = Command::new("git")
        .current_dir(path)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .args(arguments)
        .output()
        .expect("execute Git");

    assert_success("Git command", &output);

    output
}

fn assert_success(action: &str, output: &Output) {
    assert!(
        output.status.success(),
        "{action} failed\nstatus: {}\nstdout: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

struct Cleanup(PathBuf);

impl Drop for Cleanup {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.0) {
            if error.kind() != io::ErrorKind::NotFound {
                eprintln!("cleanup failed for {}: {error}", self.0.display());
            }
        }
    }
}
