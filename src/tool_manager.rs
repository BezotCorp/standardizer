use crate::{
    biome_installation::BiomeInstallation,
    resolved_tool::ResolvedTool,
    tool_source::ToolSource,
    tool_status::{ManagedToolState, ToolStatus},
};
use standardizer_common::tools::Tool;
use std::{
    env,
    error::Error,
    path::{Path, PathBuf},
};

pub(crate) struct ToolManager;

impl ToolManager {
    pub(crate) const MANAGED_TOOLS: [Tool; 1] = [Tool::Biome];

    pub(crate) fn from_name(name: &str) -> Option<Tool> {
        match name {
            "biome" => Some(Tool::Biome),
            _ => None,
        }
    }

    pub(crate) fn resolve(
        tool: Tool,
        project_root: Option<&Path>,
    ) -> Result<Option<ResolvedTool>, Box<dyn Error>> {
        for candidate in Self::candidates(tool, project_root)? {
            let version = match Self::version(tool, &candidate.0) {
                Ok(Some(version)) => version,
                Ok(None) | Err(_) => continue,
            };

            if Self::compatible_version(tool, &version)? {
                return Ok(Some(ResolvedTool::new(candidate.0, candidate.1, version)));
            }
        }

        Ok(None)
    }

    pub(crate) fn status(
        tool: Tool,
        project_root: Option<&Path>,
    ) -> Result<ToolStatus, Box<dyn Error>> {
        let active = Self::resolve(tool, project_root)?;

        let managed_path = Self::managed_path(tool)?;

        let managed_state = if !managed_path.is_file() {
            ManagedToolState::NotInstalled
        } else if Self::managed_ready(tool, &managed_path)? {
            ManagedToolState::Ready
        } else {
            ManagedToolState::Invalid
        };

        Ok(ToolStatus::new(tool, active, managed_path, managed_state))
    }

    pub(crate) fn install(tool: Tool) -> Result<PathBuf, Box<dyn Error>> {
        match tool {
            Tool::Biome => BiomeInstallation::install(false),
            Tool::Rustfmt => Err("rustfmt is not a managed external tool".into()),
        }
    }

    pub(crate) fn reinstall(tool: Tool) -> Result<PathBuf, Box<dyn Error>> {
        match tool {
            Tool::Biome => BiomeInstallation::install(true),
            Tool::Rustfmt => Err("rustfmt is not a managed external tool".into()),
        }
    }

    pub(crate) fn uninstall(tool: Tool) -> Result<bool, Box<dyn Error>> {
        match tool {
            Tool::Biome => BiomeInstallation::uninstall(),
            Tool::Rustfmt => Err("rustfmt is not a managed external tool".into()),
        }
    }

    fn managed_ready(tool: Tool, path: &Path) -> Result<bool, Box<dyn Error>> {
        match tool {
            Tool::Biome => BiomeInstallation::is_expected_version(path),
            Tool::Rustfmt => Err("rustfmt is not a managed external tool".into()),
        }
    }

    pub(crate) fn managed_path(tool: Tool) -> Result<PathBuf, Box<dyn Error>> {
        match tool {
            Tool::Biome => BiomeInstallation::managed_path(),
            Tool::Rustfmt => Err("rustfmt is not a managed external tool".into()),
        }
    }

    fn version(tool: Tool, path: &Path) -> Result<Option<semver::Version>, Box<dyn Error>> {
        match tool {
            Tool::Biome => BiomeInstallation::version(path),
            Tool::Rustfmt => Err("rustfmt is not a managed external tool".into()),
        }
    }

    fn compatible_version(tool: Tool, version: &semver::Version) -> Result<bool, Box<dyn Error>> {
        match tool {
            Tool::Biome => {
                let minimum = BiomeInstallation::minimum_version()?;

                Ok(version >= &minimum)
            }
            Tool::Rustfmt => Err("rustfmt is not a managed external tool".into()),
        }
    }

    fn candidates(
        tool: Tool,
        project_root: Option<&Path>,
    ) -> Result<Vec<(PathBuf, ToolSource)>, Box<dyn Error>> {
        match tool {
            Tool::Biome => Self::biome_candidates(project_root),
            Tool::Rustfmt => Err("rustfmt is not a managed external tool".into()),
        }
    }

    fn biome_candidates(
        project_root: Option<&Path>,
    ) -> Result<Vec<(PathBuf, ToolSource)>, Box<dyn Error>> {
        let mut candidates = Vec::new();

        if let Some(root) = project_root {
            for name in Self::project_biome_names() {
                let local = root.join("node_modules").join(".bin").join(name);

                if local.is_file() {
                    candidates.push((local, ToolSource::Project));
                }
            }
        }

        let managed = BiomeInstallation::managed_path()?;

        if managed.is_file() {
            candidates.push((managed, ToolSource::Managed));
        }

        if let Some(path) = Self::find_on_path(Self::path_biome_names()) {
            candidates.push((path, ToolSource::Path));
        }

        Ok(candidates)
    }

    fn find_on_path(names: &[&str]) -> Option<PathBuf> {
        let path = env::var_os("PATH")?;

        for directory in env::split_paths(&path) {
            for name in names {
                let candidate = directory.join(name);

                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }

        None
    }

    fn project_biome_names() -> &'static [&'static str] {
        if cfg!(target_os = "windows") {
            &["biome.exe", "biome.cmd", "biome"]
        } else {
            &["biome"]
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
