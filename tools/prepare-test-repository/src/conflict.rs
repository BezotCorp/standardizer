use std::{error::Error, fs, path::Path, process::Command};

pub(crate) fn prepare(root: &Path) -> Result<(), Box<dyn Error>> {
    let git = |args: &[&str]| crate::git::run(root, args);

    git(&["config", "user.name", "Tests"])?;
    git(&["config", "user.email", "tests@example.invalid"])?;

    let branch = git(&["symbolic-ref", "--short", "HEAD"])?;
    let branch = branch.trim();

    fs::write(root.join("example.rs"), "fn main() {}\n")?;
    git(&["add", "example.rs"])?;
    git(&["commit", "-m", "initial"])?;

    git(&["branch", "alternative"])?;

    fs::write(
        root.join("example.rs"),
        "fn main() { println!(\"first\"); }\n",
    )?;
    git(&["add", "example.rs"])?;
    git(&["commit", "-m", "first"])?;

    git(&["switch", "alternative"])?;

    fs::write(
        root.join("example.rs"),
        "fn main() { println!(\"second\"); }\n",
    )?;
    git(&["add", "example.rs"])?;
    git(&["commit", "-m", "second"])?;

    let result = Command::new("git")
        .current_dir(root)
        .args(["merge", branch])
        .output()?;

    if result.status.success() {
        return Err("expected merge conflict".into());
    }

    if git(&["ls-files", "-u"])?.is_empty() {
        return Err("no unresolved Git entries".into());
    }

    Ok(())
}
