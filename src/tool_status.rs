use crate::resolved_tool::ResolvedTool;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub(crate) struct ToolStatus {
    active: Option<ResolvedTool>,
    active_version: Option<String>,
    managed_path: PathBuf,
    managed_installed: bool,
}

impl ToolStatus {
    pub(crate) fn new(
        active: Option<ResolvedTool>,
        active_version: Option<String>,
        managed_path: PathBuf,
        managed_installed: bool,
    ) -> Self {
        Self {
            active,
            active_version,
            managed_path,
            managed_installed,
        }
    }

    pub(crate) fn active(&self) -> Option<&ResolvedTool> {
        self.active.as_ref()
    }

    pub(crate) fn active_version(&self) -> Option<&str> {
        self.active_version.as_deref()
    }

    pub(crate) fn managed_path(&self) -> &std::path::Path {
        &self.managed_path
    }

    pub(crate) fn managed_installed(&self) -> bool {
        self.managed_installed
    }
}
