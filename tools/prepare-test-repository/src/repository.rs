use std::{
    env,
    error::Error,
    fs, io,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

fn create() -> Result<PathBuf, Box<dyn Error>> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();

    for _ in 0..100 {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);

        let path = env::temp_dir().join(format!(
            "standardizer-test-{}-{timestamp}-{id}",
            std::process::id()
        ));

        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                continue;
            }
            Err(error) => return Err(error.into()),
        }
    }

    Err("cannot create a unique temporary directory".into())
}

fn initialize(path: &Path) -> Result<(), Box<dyn Error>> {
    let output = Command::new("git")
        .args(["init", "-q"])
        .current_dir(path)
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "git init failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    Ok(())
}

pub(crate) fn prepare(
    scenario: fn(&Path) -> Result<(), Box<dyn Error>>,
) -> Result<PathBuf, Box<dyn Error>> {
    let path = create()?;

    let result = initialize(&path).and_then(|()| scenario(&path));

    if let Err(error) = result {
        if let Err(cleanup_error) = fs::remove_dir_all(&path) {
            return Err(format!(
                "{error}; cleanup failed for {}: {cleanup_error}",
                path.display()
            )
            .into());
        }

        return Err(error);
    }

    Ok(path)
}
