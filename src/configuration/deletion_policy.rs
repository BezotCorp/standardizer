#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum DeletionPolicy {
    #[default]
    Automatic,
    Ignore,
}
