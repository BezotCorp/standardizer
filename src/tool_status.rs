use crate::{
    resolved_tool::ResolvedTool,
    tool_id::ToolId,
};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ManagedToolState {
    NotInstalled,
    Ready,
    Invalid,
}

impl ManagedToolState {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::NotInstalled => "Not installed",
            Self::Ready => "Ready",
            Self::Invalid => "Invalid",
        }
    }

    pub(crate) fn is_installed(self) -> bool {
        !matches!(self, Self::NotInstalled)
    }

    pub(crate) fn is_ready(self) -> bool {
        matches!(self, Self::Ready)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ToolStatus {
    tool: ToolId,
    active: Option<ResolvedTool>,
    managed_path: PathBuf,
    managed_state: ManagedToolState,
}

impl ToolStatus {
    pub(crate) fn new(
        tool: ToolId,
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
