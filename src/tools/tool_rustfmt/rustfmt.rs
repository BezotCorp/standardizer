use crate::{cargo_metadata::CargoMetadata, goals::ConformanceIntent};
use std::{error::Error, path::Path, process::Command};

pub(crate) struct Rustfmt;

impl Rustfmt {
    pub(crate) fn verify(root: &Path) -> Result<bool, Box<dyn Error>> {
        let output = match Command::new("rustfmt")
            .current_dir(root)
            .arg("--version")
            .output()
        {
            Ok(output) => output,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                eprintln!("standardizer: rustfmt absent, fichiers correspondants ignorés");
                return Ok(false);
            }
            Err(error) => return Err(error.into()),
        };

        if !output.status.success() {
            return Err(format!(
                "rustfmt indisponible : {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )
            .into());
        }

        Ok(true)
    }

    pub(crate) fn execute(
        root: &Path,
        files: &[&Path],
        intent: ConformanceIntent,
    ) -> Result<bool, Box<dyn Error>> {
        if files.is_empty() {
            return Ok(true);
        }

        let metadata = CargoMetadata::load(root)?;
        let mut passed = true;

        for path in files {
            let absolute = root.join(path);
            let mut command = Command::new("rustfmt");

            command.current_dir(root);

            if let Some(edition) = metadata.edition(&absolute) {
                command.args(["--edition", edition]);
            }

            command.args(["--config", "skip_children=true"]);

            if intent == ConformanceIntent::CheckConformance {
                command.arg("--check");
            }

            passed &= command.arg(&absolute).status()?.success();
        }

        Ok(passed)
    }
}
