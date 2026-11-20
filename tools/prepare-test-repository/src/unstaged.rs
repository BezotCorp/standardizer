use std::{error::Error, fs, path::Path};

pub(crate) fn prepare(root: &Path) -> Result<(), Box<dyn Error>> {
    fs::write(root.join("example.rs"), "fn main() {}\n")?;

    crate::git::run(root, &["add", "example.rs"])?;

    fs::write(
        root.join("example.rs"),
        "fn main(){println!(\"modified\");}\n",
    )?;

    Ok(())
}
