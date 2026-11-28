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
