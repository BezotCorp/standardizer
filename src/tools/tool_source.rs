#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ToolSource {
    Project,
    Managed,
    Path,
}

impl ToolSource {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::Managed => "standardizer",
            Self::Path => "system",
        }
    }
}
