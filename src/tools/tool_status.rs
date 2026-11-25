use crate::resolved_tool::ResolvedTool;
use crate::tools::ManagedToolState;
use standardizer_common::tools::Tool;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub(crate) struct ToolStatus {
    tool: Tool,
    active: Option<ResolvedTool>,
    managed_path: PathBuf,
    managed_state: ManagedToolState,
}

impl ToolStatus {
    pub(crate) fn new(
        tool: Tool,
        active: Option<ResolvedTool>,
        managed_path: PathBuf,
        managed_state: ManagedToolState,
    ) -> Self {
        Self {
            tool,
            active,
            managed_path,
            managed_state,
        }
    }

    pub(crate) fn tool(&self) -> Tool {
        self.tool
    }

    pub(crate) fn active(&self) -> Option<&ResolvedTool> {
        self.active.as_ref()
    }

    pub(crate) fn active_version(&self) -> Option<String> {
        self.active.as_ref().map(|tool| tool.version().to_string())
    }

    pub(crate) fn managed_path(&self) -> &std::path::Path {
        &self.managed_path
    }

    pub(crate) fn managed_state(&self) -> ManagedToolState {
        self.managed_state
    }

    pub(crate) fn managed_installed(&self) -> bool {
        self.managed_state.is_installed()
    }

    pub(crate) fn managed_ready(&self) -> bool {
        self.managed_state.is_ready()
    }
}
