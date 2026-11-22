use crate::{
    resolved_tool::ResolvedTool,
    tool_id::ToolId,
};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub(crate) struct ToolStatus {
    tool: ToolId,
    active: Option<ResolvedTool>,
    managed_path: PathBuf,
    managed_installed: bool,
}

impl ToolStatus {
    pub(crate) fn new(
        tool: ToolId,
        active: Option<ResolvedTool>,
        managed_path: PathBuf,
        managed_installed: bool,
    ) -> Self {
        Self {
            tool,
            active,
            managed_path,
            managed_installed,
        }
    }

    pub(crate) fn tool(&self) -> ToolId {
        self.tool
    }

    pub(crate) fn active(&self) -> Option<&ResolvedTool> {
        self.active.as_ref()
    }

    pub(crate) fn active_version(&self) -> Option<String> {
        self.active
            .as_ref()
            .map(|tool| tool.version().to_string())
    }

    pub(crate) fn managed_path(&self) -> &std::path::Path {
        &self.managed_path
    }

    pub(crate) fn managed_installed(&self) -> bool {
        self.managed_installed
    }
}
