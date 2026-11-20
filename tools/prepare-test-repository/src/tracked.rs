use std::{error::Error, path::Path};

pub(crate) fn prepare(root: &Path) -> Result<(), Box<dyn Error>> {
    crate::untracked::prepare(root)?;
    crate::git::run(root, &["add", "--all"])?;
    Ok(())
}
