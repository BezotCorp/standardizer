use crate::configuration::{
    DeletionPolicy, RepositoryConfiguration, tests::test_repository::TestRepository,
};
use std::error::Error;

const TEAM: &str = ".standardizer/team.json";
const DEV: &str = ".standardizer/dev.json";
const IGNORE: &str = r#"{"deletion":"ignore"}"#;

#[test]
fn missing_configurations_use_automatic_policy() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;

    let configuration = RepositoryConfiguration::load(repository.root())?;

    assert_eq!(configuration.deletion(), DeletionPolicy::Automatic);

    Ok(())
}

#[test]
fn empty_configurations_use_automatic_policy() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;

    repository.write(TEAM, "{}")?;
    repository.write(DEV, "{}")?;

    let configuration = RepositoryConfiguration::load(repository.root())?;

    assert_eq!(configuration.deletion(), DeletionPolicy::Automatic);

    Ok(())
}

#[test]
fn team_can_disable_automatic_deletions() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;

    repository.write(TEAM, IGNORE)?;

    let configuration = RepositoryConfiguration::load(repository.root())?;

    assert_eq!(configuration.deletion(), DeletionPolicy::Ignore);

    Ok(())
}

#[test]
fn developer_can_disable_automatic_deletions() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;

    repository.write(DEV, IGNORE)?;

    let configuration = RepositoryConfiguration::load(repository.root())?;

    assert_eq!(configuration.deletion(), DeletionPolicy::Ignore);

    Ok(())
}

#[test]
fn missing_team_property_uses_developer_property() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;

    repository.write(TEAM, "{}")?;
    repository.write(DEV, IGNORE)?;

    let configuration = RepositoryConfiguration::load(repository.root())?;

    assert_eq!(configuration.deletion(), DeletionPolicy::Ignore);

    Ok(())
}

#[test]
fn team_property_is_preserved() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;

    repository.write(TEAM, IGNORE)?;
    repository.write(DEV, "{}")?;

    let configuration = RepositoryConfiguration::load(repository.root())?;

    assert_eq!(configuration.deletion(), DeletionPolicy::Ignore);

    Ok(())
}

#[test]
fn both_configurations_can_disable_automatic_deletions() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;

    repository.write(TEAM, IGNORE)?;
    repository.write(DEV, IGNORE)?;

    let configuration = RepositoryConfiguration::load(repository.root())?;

    assert_eq!(configuration.deletion(), DeletionPolicy::Ignore);

    Ok(())
}

#[test]
fn automatic_cannot_be_explicitly_configured() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;

    repository.write(TEAM, r#"{"deletion":"automatic"}"#)?;

    let result = RepositoryConfiguration::load(repository.root());

    assert!(result.is_err());

    Ok(())
}

#[test]
fn invalid_policy_values_are_rejected() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;

    for content in [
        r#"{"deletion":"unknown"}"#,
        r#"{"deletion":"IGNORE"}"#,
        r#"{"deletion":true}"#,
        r#"{"deletion":null}"#,
        r#"{"deletion":42}"#,
        r#"{"deletion":[]}"#,
        r#"{"deletion":{}}"#,
    ] {
        repository.write(TEAM, content)?;

        assert!(
            RepositoryConfiguration::load(repository.root()).is_err(),
            "{content}"
        );
    }

    Ok(())
}

#[test]
fn invalid_json_is_rejected() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;

    repository.write(TEAM, "{")?;

    assert!(RepositoryConfiguration::load(repository.root()).is_err());

    Ok(())
}

#[test]
fn configuration_must_be_an_object() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;

    for content in ["null", "[]", "true", "42"] {
        repository.write(TEAM, content)?;

        assert!(
            RepositoryConfiguration::load(repository.root()).is_err(),
            "{content}"
        );
    }

    Ok(())
}

#[test]
fn unknown_properties_are_rejected() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;

    repository.write(TEAM, r#"{"unknown":true}"#)?;

    assert!(RepositoryConfiguration::load(repository.root()).is_err());

    Ok(())
}

#[test]
fn invalid_developer_configuration_is_rejected() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;

    repository.write(TEAM, IGNORE)?;
    repository.write(DEV, r#"{"deletion":null}"#)?;

    assert!(RepositoryConfiguration::load(repository.root()).is_err());

    Ok(())
}

#[test]
fn loading_configuration_does_not_modify_repository() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;

    repository.write(TEAM, IGNORE)?;
    repository.write(DEV, "{}")?;

    let before = repository.git(&["status", "--porcelain", "--untracked-files=all"])?;

    let configuration = RepositoryConfiguration::load(repository.root())?;

    let after = repository.git(&["status", "--porcelain", "--untracked-files=all"])?;

    assert_eq!(configuration.deletion(), DeletionPolicy::Ignore);
    assert_eq!(before, after);

    Ok(())
}
