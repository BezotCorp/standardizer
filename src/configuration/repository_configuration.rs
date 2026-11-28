use crate::configuration::{DeletionPolicy, DeveloperConfiguration, settings::Settings};
use std::{error::Error, path::Path};

pub(crate) struct RepositoryConfiguration {
    settings: Settings,
}

impl RepositoryConfiguration {
    const TEAM_PATH: &str = ".standardizer/team.json";

    pub(crate) fn load(root: &Path) -> Result<Self, Box<dyn Error>> {
        let team = Settings::load(&root.join(Self::TEAM_PATH))?;
        let developer = Settings::load(&root.join(DeveloperConfiguration::PATH))?;

        Ok(Self {
            settings: team.merge(developer),
        })
    }

    pub(crate) fn deletion(&self) -> DeletionPolicy {
        self.settings.deletion()
    }
}
