use std::{error::Error, fs, path::Path};

pub(crate) fn prepare(root: &Path) -> Result<(), Box<dyn Error>> {
    crate::git::run(root, &["config", "user.name", "Tests"])?;
    crate::git::run(root, &["config", "user.email", "tests@example.invalid"])?;

    fs::write(root.join("example.rs"), "fn main() {}\n")?;
    crate::git::run(root, &["add", "example.rs"])?;
    crate::git::run(root, &["commit", "-m", "initial"])?;

    fs::write(
        root.join("example.rs"),
        "fn main(){println!(\"staged\");}\n",
    )?;
    crate::git::run(root, &["add", "example.rs"])?;

    fs::write(
        root.join("example.rs"),
        "fn main(){println!(\"working\");}\n",
    )?;

    Ok(())
}
