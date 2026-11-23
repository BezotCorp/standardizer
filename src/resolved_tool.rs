use crate::tool_source::ToolSource;
use semver::Version;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub(crate) struct ResolvedTool {
    path: PathBuf,
    source: ToolSource,
    version: Version,
}

impl ResolvedTool {
    pub(crate) fn new(path: PathBuf, source: ToolSource, version: Version) -> Self {
        Self {
            path,
            source,
            version,
        }
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
