use crate::{
    tool_id::ToolId,
    tool_source::ToolSource,
};
use semver::Version;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub(crate) struct ResolvedTool {
    tool: ToolId,
    path: PathBuf,
    source: ToolSource,
    version: Version,
}

impl ResolvedTool {
    pub(crate) fn new(
        tool: ToolId,
        path: PathBuf,
        source: ToolSource,
        version: Version,
    ) -> Self {
        Self {
            tool,
            path,
            source,
            version,
        }
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

    pub(crate) fn version(&self) -> &Version {
        &self.version
    }
}
