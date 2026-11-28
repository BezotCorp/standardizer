use crate::configuration::{DeveloperConfiguration, tests::test_repository::TestRepository};
use std::{error::Error, fs};

const DEV: &str = ".standardizer/dev.json";
const TEAM: &str = ".standardizer/team.json";

#[test]
fn developer_configuration_is_locally_excluded() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;

    DeveloperConfiguration::protect(repository.root())?;

    repository.write(DEV, "{}")?;
    repository.write(TEAM, "{}")?;

    let status = repository.git(&["status", "--porcelain", "--untracked-files=all"])?;

    assert!(!status.contains(DEV));
    assert!(status.contains(TEAM));
    assert!(!repository.root().join(".gitignore").exists());

    Ok(())
}

#[test]
fn existing_exclusions_are_preserved() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;

    repository.write(".git/info/exclude", "# existing\n/local.txt")?;

    DeveloperConfiguration::protect(repository.root())?;

    let content = fs::read_to_string(repository.root().join(".git/info/exclude"))?;

    assert_eq!(content, "# existing\n/local.txt\n/.standardizer/dev.json\n");

    Ok(())
}

#[test]
fn protection_is_idempotent() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;

    DeveloperConfiguration::protect(repository.root())?;
    DeveloperConfiguration::protect(repository.root())?;

    let content = fs::read_to_string(repository.root().join(".git/info/exclude"))?;

    assert_eq!(
        content
            .lines()
            .filter(|line| *line == "/.standardizer/dev.json")
            .count(),
        1
    );

    Ok(())
}

#[test]
fn newly_staged_developer_configuration_is_untracked() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;

    repository.write(DEV, "{}")?;
    repository.git(&["add", "--force", "--", DEV])?;

    DeveloperConfiguration::protect(repository.root())?;

    let tracked = repository.git(&["ls-files", "--", DEV])?;

    assert!(tracked.is_empty());
    assert_eq!(fs::read_to_string(repository.root().join(DEV))?, "{}");

    Ok(())
}

#[test]
fn committed_developer_configuration_is_untracked() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;

    repository.write(DEV, "{}")?;
    repository.git(&["add", "--force", "--", DEV])?;
    repository.git(&["commit", "--quiet", "-m", "chore: initial configuration"])?;

    let head = repository.git(&["rev-parse", "HEAD"])?;

    repository.write(DEV, r#"{"deletion":"ignore"}"#)?;

    DeveloperConfiguration::protect(repository.root())?;

    let staged = repository.git(&["diff", "--cached", "--name-status"])?;

    assert_eq!(staged.trim(), "D\t.standardizer/dev.json");

    assert_eq!(
        fs::read_to_string(repository.root().join(DEV))?,
        r#"{"deletion":"ignore"}"#
    );

    assert_eq!(repository.git(&["rev-parse", "HEAD"])?, head);

    Ok(())
}

#[test]
fn unrelated_index_entries_are_preserved() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;

    repository.write("other.txt", "other")?;
    repository.write(TEAM, "{}")?;
    repository.write(DEV, "{}")?;

    repository.git(&["add", "--force", "--", "other.txt", TEAM, DEV])?;

    let before = repository.git(&["ls-files", "--stage", "--", "other.txt", TEAM])?;

    DeveloperConfiguration::protect(repository.root())?;

    let after = repository.git(&["ls-files", "--stage", "--", "other.txt", TEAM])?;

    assert_eq!(before, after);

    Ok(())
}

#[test]
fn protection_does_not_create_configuration_files() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;

    DeveloperConfiguration::protect(repository.root())?;

    assert!(!repository.root().join(DEV).exists());
    assert!(!repository.root().join(TEAM).exists());

    Ok(())
}

#[test]
fn protection_preserves_untracked_developer_configuration() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;

    repository.write(DEV, r#"{"deletion":"ignore"}"#)?;

    DeveloperConfiguration::protect(repository.root())?;

    assert_eq!(
        fs::read_to_string(repository.root().join(DEV))?,
        r#"{"deletion":"ignore"}"#
    );

    let status = repository.git(&["status", "--porcelain", "--untracked-files=all"])?;

    assert!(!status.contains(DEV));

    Ok(())
}
