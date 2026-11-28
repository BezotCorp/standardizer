use std::{
    env,
    error::Error,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::{self, Command},
};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let arguments: Vec<_> = env::args_os().skip(1).collect();

    if arguments.iter().any(|argument| argument == "--version") {
        println!("rustfmt 1.9.0");
        return Ok(());
    }

    let file = arguments
        .last()
        .map(PathBuf::from)
        .ok_or("missing formatter input")?;

    if !file.is_file() {
        return Err(format!("missing input file: {}", file.display(),).into());
    }

    if arguments.iter().any(|argument| argument == "--check") {
        return Ok(());
    }

    OpenOptions::new()
        .append(true)
        .open(&file)?
        .write_all(b"// formatted\n")?;

    let scenario = env::var("STANDARDIZER_TEST_SCENARIO").unwrap_or_else(|_| "normal".to_owned());

    let name = file
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("invalid formatter input name")?;

    match (scenario.as_str(), name) {
        ("change_before", "a.rs") => {
            fs::write("b.rs", "fn external() {}\n")?;
        }
        ("change_after", "b.rs") => {
            fs::write("a.rs", "fn external() {}\n")?;
        }
        ("delete_before", "a.rs") => {
            fs::remove_file("b.rs")?;
        }
        ("delete_after", "b.rs") => {
            fs::remove_file("a.rs")?;
        }
        ("change_unrouted", "a.rs") => {
            fs::write("notes.txt", "external\n")?;
        }
        ("change_index", "b.rs") => {
            fs::write("a.rs", "fn external() {}\n")?;

            let status = Command::new("git").args(["add", "--", "a.rs"]).status()?;

            if !status.success() {
                return Err("external Git staging failed".into());
            }
        }
        ("change_mode", "a.rs") => {
            change_mode(Path::new("b.rs"))?;
        }
        ("fail", _) => {
            return Err("simulated formatter failure".into());
        }
        _ => {}
    }

    Ok(())
}

#[cfg(unix)]
fn change_mode(path: &Path) -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)?.permissions();

    permissions.set_mode(permissions.mode() | 0o111);

    fs::set_permissions(path, permissions)?;

    Ok(())
}

#[cfg(not(unix))]
fn change_mode(_: &Path) -> Result<(), Box<dyn Error>> {
    Err("executable mode is not supported by this test".into())
}
