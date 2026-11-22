#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ToolId {
    Biome,
}

impl ToolId {
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        match name {
            "biome" => Some(Self::Biome),
            _ => None,
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Biome => "biome",
        }
    }

    pub(crate) fn display_name(self) -> &'static str {
        match self {
            Self::Biome => "Biome",
        }
    }
}
