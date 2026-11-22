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
        match tool {
            ToolId::Biome => Self::resolve_biome(project_root),
        }
    }

    pub(crate) fn status(
        tool: ToolId,
        project_root: Option<&Path>,
    ) -> Result<ToolStatus, Box<dyn Error>> {
        let active = Self::resolve(tool, project_root)?;

        let active_version = match active.as_ref() {
            Some(resolved) => {
                BiomeInstallation::version(resolved.path())?
            }
            None => None,
        };

        let managed_path = BiomeInstallation::managed_path()?;
        let managed_installed = managed_path.is_file();

        Ok(ToolStatus::new(
            active,
            active_version,
            managed_path,
            managed_installed,
        ))
    }

    pub(crate) fn install(
        tool: ToolId,
    ) -> Result<PathBuf, Box<dyn Error>> {
        match tool {
            ToolId::Biome => BiomeInstallation::install(false),
        }
    }

    pub(crate) fn reinstall(
        tool: ToolId,
    ) -> Result<PathBuf, Box<dyn Error>> {
        match tool {
            ToolId::Biome => BiomeInstallation::install(true),
        }
    }

    pub(crate) fn uninstall(
        tool: ToolId,
    ) -> Result<bool, Box<dyn Error>> {
        match tool {
            ToolId::Biome => BiomeInstallation::uninstall(),
        }
    }

    fn resolve_biome(
        project_root: Option<&Path>,
    ) -> Result<Option<ResolvedTool>, Box<dyn Error>> {
        if let Some(root) = project_root {
            let local = root
                .join("node_modules")
                .join(".bin")
                .join(Self::project_biome_name());

            if local.is_file() {
                return Ok(Some(ResolvedTool::new(
                    ToolId::Biome,
                    local,
                    ToolSource::Project,
                )));
            }
        }

        let managed = BiomeInstallation::managed_path()?;

        if managed.is_file() {
            return Ok(Some(ResolvedTool::new(
                ToolId::Biome,
                managed,
                ToolSource::Managed,
            )));
        }

        if let Some(path) = Self::find_on_path() {
            return Ok(Some(ResolvedTool::new(
                ToolId::Biome,
                path,
                ToolSource::Path,
            )));
        }

        Ok(None)
    }

    fn find_on_path() -> Option<PathBuf> {
        let path = env::var_os("PATH")?;

        for directory in env::split_paths(&path) {
            for name in Self::path_biome_names() {
                let candidate = directory.join(name);

                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }

        None
    }

    fn project_biome_name() -> &'static str {
        if cfg!(target_os = "windows") {
            "biome.cmd"
        } else {
            "biome"
        }
    }

    fn path_biome_names() -> &'static [&'static str] {
        if cfg!(target_os = "windows") {
            &["biome.exe", "biome.cmd", "biome"]
        } else {
            &["biome"]
        }
    }
}
