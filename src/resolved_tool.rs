use crate::{tool_id::ToolId, tool_source::ToolSource};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub(crate) struct ResolvedTool {
    tool: ToolId,
    path: PathBuf,
    source: ToolSource,
}

impl ResolvedTool {
    pub(crate) fn new(tool: ToolId, path: PathBuf, source: ToolSource) -> Self {
        Self { tool, path, source }
    }

    pub(crate) fn tool(&self) -> ToolId {
        self.tool
    }

    pub(crate) fn path(&self) -> &std::path::Path {
        &self.path
    }

    pub(crate) fn source(&self) -> ToolSource {
        self.source
    }
}
