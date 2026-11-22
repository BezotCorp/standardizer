use crate::{
    biome_installation::BiomeInstallation,
    resolved_tool::ResolvedTool,
    tool_id::ToolId,
    tool_source::ToolSource,
    tool_status::ToolStatus,
};
use std::{
    env,
    error::Error,
    path::{Path, PathBuf},
};

pub(crate) struct ToolManager;

impl ToolManager {
    pub(crate) fn resolve(
        tool: ToolId,
        project_root: Option<&Path>,
    ) -> Result<Option<ResolvedTool>, Box<dyn Error>> {
        for candidate in
            Self::candidates(tool, project_root)?
        {
            let version =
                match Self::version(tool, &candidate.0)
                {
                    Ok(Some(version)) => version,
                    Ok(None) | Err(_) => continue,
                };

            if Self::compatible_version(
                tool,
                &version,
            )? {
                return Ok(Some(
                    ResolvedTool::new(
                        tool,
                        candidate.0,
                        candidate.1,
                        version,
                    ),
                ));
            }
        }

        Ok(None)
    }

    pub(crate) fn status(
        tool: ToolId,
        project_root: Option<&Path>,
    ) -> Result<ToolStatus, Box<dyn Error>> {
        let active =
            Self::resolve(tool, project_root)?;

        let managed_path =
            Self::managed_path(tool)?;

        let managed_installed =
            managed_path.is_file();

        Ok(
            ToolStatus::new(
                tool,
                active,
                managed_path,
                managed_installed,
            ),
        )
    }

    pub(crate) fn install(
        tool: ToolId,
    ) -> Result<PathBuf, Box<dyn Error>> {
        match tool {
            ToolId::Biome => {
                BiomeInstallation::install(false)
            }
        }
    }

    pub(crate) fn reinstall(
        tool: ToolId,
    ) -> Result<PathBuf, Box<dyn Error>> {
        match tool {
            ToolId::Biome => {
                BiomeInstallation::install(true)
            }
        }
    }

    pub(crate) fn uninstall(
        tool: ToolId,
    ) -> Result<bool, Box<dyn Error>> {
        match tool {
            ToolId::Biome => {
                BiomeInstallation::uninstall()
            }
        }
    }

    pub(crate) fn managed_path(
        tool: ToolId,
    ) -> Result<PathBuf, Box<dyn Error>> {
        match tool {
            ToolId::Biome => {
                BiomeInstallation::managed_path()
            }
        }
    }

    fn version(
        tool: ToolId,
        path: &Path,
    ) -> Result<
        Option<semver::Version>,
        Box<dyn Error>,
    > {
        match tool {
            ToolId::Biome => {
                BiomeInstallation::version(path)
            }
        }
    }

    fn compatible_version(
        tool: ToolId,
        version: &semver::Version,
    ) -> Result<bool, Box<dyn Error>> {
        match tool {
            ToolId::Biome => {
                let minimum = semver::Version::parse(
                    BiomeInstallation::MINIMUM_VERSION,
                )?;

                Ok(version >= &minimum)
            }
        }
    }

    fn candidates(
        tool: ToolId,
        project_root: Option<&Path>,
    ) -> Result<
        Vec<(PathBuf, ToolSource)>,
        Box<dyn Error>,
    > {
        match tool {
            ToolId::Biome => {
                Self::biome_candidates(project_root)
            }
        }
    }

    fn biome_candidates(
        project_root: Option<&Path>,
    ) -> Result<
        Vec<(PathBuf, ToolSource)>,
        Box<dyn Error>,
    > {
        let mut candidates = Vec::new();

        if let Some(root) = project_root {
            for name in
                Self::project_biome_names()
            {
                let local = root
                    .join("node_modules")
                    .join(".bin")
                    .join(name);

                if local.is_file() {
                    candidates.push((
                        local,
                        ToolSource::Project,
                    ));
                }
            }
        }

        let managed =
            BiomeInstallation::managed_path()?;

        if managed.is_file() {
            candidates.push((
                managed,
                ToolSource::Managed,
            ));
        }

        if let Some(path) =
            Self::find_on_path(
                Self::path_biome_names(),
            )
        {
            candidates.push((
                path,
                ToolSource::Path,
            ));
        }

        Ok(candidates)
    }

    fn find_on_path(
        names: &[&str],
    ) -> Option<PathBuf> {
        let path = env::var_os("PATH")?;

        for directory in env::split_paths(&path) {
            for name in names {
                let candidate =
                    directory.join(name);

                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }

        None
    }

    fn project_biome_names(
    ) -> &'static [&'static str] {
        if cfg!(target_os = "windows") {
            &[
                "biome.exe",
                "biome.cmd",
                "biome",
            ]
        } else {
            &["biome"]
        }
    }

    fn path_biome_names(
    ) -> &'static [&'static str] {
        if cfg!(target_os = "windows") {
            &[
                "biome.exe",
                "biome.cmd",
                "biome",
            ]
        } else {
            &["biome"]
        }
    }
}
