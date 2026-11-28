use crate::{
    git_repository::GitRepository,
    goals::{Conformance, ConformanceIntent},
};
use std::{error::Error, ffi::OsString};

pub(crate) struct ConformancePipeline;

impl ConformancePipeline {
    pub(crate) fn run(arguments: impl Iterator<Item = OsString>) -> Result<(), Box<dyn Error>> {
        let intent = ConformanceIntent::from_arguments(arguments)?;
        let repository = GitRepository::discover()?;
        let configuration = crate::configuration::RepositoryConfiguration::load(repository.root())?;

        if intent == crate::goals::ConformanceIntent::FixConformance {
            crate::configuration::DeveloperConfiguration::protect(repository.root())?;
        }
        let files = repository.candidates()?;

        let expected = Conformance::execute(&repository, &files, intent)?;
        repository.stage_changed(&files, &expected, intent)?;

        if intent == crate::goals::ConformanceIntent::FixConformance
            && configuration.deletion() == crate::configuration::DeletionPolicy::Automatic
        {
            repository.stage_deletions()?;
        }

        Ok(())
    }
}
