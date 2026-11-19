use crate::conformance_intent::ConformanceIntent;
use std::{error::Error, ffi::OsString, path::Path, process::Command};

pub(crate) struct Biome;

impl Biome {
    fn executable(root: &Path) -> OsString {
        let binary = if cfg!(windows) { "biome.cmd" } else { "biome" };
        let local = root.join("node_modules/.bin").join(binary);
        if local.is_file() {
            local.into_os_string()
        } else {
            OsString::from(binary)
        }
    }

    pub(crate) fn verify(root: &Path) -> Result<(), Box<dyn Error>> {
        let output = Command::new(Self::executable(root))
            .current_dir(root)
            .arg("--version")
            .output()
            .map_err(|error| format!("impossible de lancer biome : {error} ; installez Biome ou rendez-le accessible dans PATH"))?;
        if !output.status.success() {
            return Err(format!(
                "biome indisponible : {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )
            .into());
        }
        Ok(())
    }

    pub(crate) fn execute(
        root: &Path,
        files: &[&Path],
        intent: ConformanceIntent,
    ) -> Result<bool, Box<dyn Error>> {
        if files.is_empty() {
            return Ok(true);
        }

        let executable = Self::executable(root);

        let mut passed = true;

        for chunk in files.chunks(64) {
            let mut command = Command::new(&executable);

            command.current_dir(root).args([
                "check",
                "--files-ignore-unknown=true",
                "--no-errors-on-unmatched",
            ]);

            if intent == ConformanceIntent::FixConformance {
                command.arg("--write");
            }

            passed &= command
                .arg("--")
                .args(chunk.iter().map(|path| root.join(path)))
                .status()?
                .success();
        }

        Ok(passed)
    }
}
