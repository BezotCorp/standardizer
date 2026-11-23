use crate::{
    biome_installation::BiomeInstallation, conformance_intent::ConformanceIntent,
    tool_manager::ToolManager,
};
use standardizer_common::tools::Tool;
use std::{
    error::Error,
    io::{self, IsTerminal, Write},
    path::{Path, PathBuf},
    process::Command,
};

pub(crate) struct Biome;

impl Biome {
    pub(crate) fn verify(root: &Path) -> Result<bool, Box<dyn Error>> {
        if let Some(executable) = Self::executable(root)? {
            return Self::verify_executable(&executable);
        }

        eprintln!("standardizer: aucune version compatible de Biome trouvée");

        let minimum_version = BiomeInstallation::minimum_version()?;

        eprintln!("standardizer: Biome >= {} requis", minimum_version,);

        if io::stdin().is_terminal() && io::stdout().is_terminal() {
            let managed_version = BiomeInstallation::managed_version()?;

            print!(
                "Installer Biome {} avec Standardizer maintenant ? [y/N] ",
                managed_version,
            );

            io::stdout().flush()?;

            let mut answer = String::new();

            io::stdin().read_line(&mut answer)?;

            let accepted = matches!(
                answer.trim().to_ascii_lowercase().as_str(),
                "y" | "yes" | "o" | "oui",
            );

            if accepted {
                let executable = ToolManager::install(Tool::Biome)?;

                if Self::verify_executable(&executable)? {
                    return Ok(true);
                }

                return Err("Biome was installed but could not be verified".into());
            }
        } else {
            eprintln!(
                "standardizer: CI/non-interactif : \
                 `standardizer tools install biome`"
            );
        }

        Ok(false)
    }

    pub(crate) fn execute(
        root: &Path,
        files: &[&Path],
        intent: ConformanceIntent,
    ) -> Result<bool, Box<dyn Error>> {
        if files.is_empty() {
            return Ok(true);
        }

        let executable =
            Self::executable(root)?.ok_or("compatible Biome is required but unavailable")?;

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

    fn executable(root: &Path) -> Result<Option<PathBuf>, Box<dyn Error>> {
        Ok(ToolManager::resolve(Tool::Biome, Some(root))?.map(|tool| tool.path().to_path_buf()))
    }

    fn verify_executable(executable: &Path) -> Result<bool, Box<dyn Error>> {
        BiomeInstallation::is_compatible(executable)
    }
}
