use crate::support::TestRepository;
use std::error::Error;

#[test]
fn untracked_files_are_preserved() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::from_scenario("untracked")?;
    let before = repository.status()?;
    let source = repository.read("example.rs")?;

    repository.run_real("fix")?;

    assert_eq!(repository.status()?, before);
    assert_eq!(repository.read("example.rs")?, source);

    Ok(())
}

#[test]
fn staged_files_are_formatted() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::from_scenario("tracked")?;
    let before = repository.index("example.rs")?;
    let text = repository.read("example.txt")?;

    repository.run_real("fix")?;

    let after = repository.read("example.rs")?;

    assert_ne!(after, before);
    assert_eq!(repository.index("example.rs")?, after);
    assert_eq!(repository.read("example.txt")?, text);
    assert_eq!(repository.index("example.txt")?, text);

    repository.run_real("check")?;

    Ok(())
}

#[test]
fn unstaged_changes_are_preserved() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::from_scenario("unstaged")?;
    let index = repository.index("example.rs")?;
    let source = repository.read("example.rs")?;
    let status = repository.status()?;

    repository.run_real("fix")?;

    assert_eq!(repository.index("example.rs")?, index);
    assert_eq!(repository.read("example.rs")?, source);
    assert_eq!(repository.status()?, status);

    Ok(())
}

#[test]
fn partially_staged_changes_are_preserved() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::from_scenario("partial")?;
    let index = repository.index("example.rs")?;
    let source = repository.read("example.rs")?;
    let status = repository.status()?;

    repository.run_real("fix")?;

    assert_eq!(repository.index("example.rs")?, index);
    assert_eq!(repository.read("example.rs")?, source);
    assert_eq!(repository.status()?, status);

    Ok(())
}

#[test]
fn unresolved_conflicts_are_preserved() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::from_scenario("conflict")?;
    let index = repository.git(&["ls-files", "--unmerged"])?;
    let source = repository.read("example.rs")?;

    let _ = repository.run_real("fix");

    assert_eq!(repository.git(&["ls-files", "--unmerged"])?, index);
    assert_eq!(repository.read("example.rs")?, source);

    Ok(())
}

#[test]
fn invalid_source_is_not_staged_again() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::from_scenario("invalid")?;
    let index = repository.index("example.rs")?;

    assert!(repository.run_real("fix").is_err());
    assert_eq!(repository.index("example.rs")?, index);

    Ok(())
}

#[test]
fn unstaged_deletion_is_automatically_staged() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::from_scenario("deletion-unstaged")?;

    repository.run_real("fix")?;

    assert!(!repository.exists("example.rs"));
    assert!(
        repository
            .git(&["ls-files", "--", "example.rs"])?
            .is_empty()
    );
    assert!(repository.status()?.contains("D  example.rs"));

    Ok(())
}

#[test]
fn staged_deletion_is_preserved() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::from_scenario("deletion-staged")?;

    repository.run_real("fix")?;

    assert!(!repository.exists("example.rs"));
    assert!(repository.status()?.contains("D  example.rs"));

    Ok(())
}

#[test]
fn unstaged_modification_prevents_deletion_staging() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::from_scenario("deletion-with-modification")?;
    let index = repository.index("example.rs")?;
    let source = repository.read("other.rs")?;

    repository.run_real("fix")?;

    assert_eq!(repository.index("example.rs")?, index);
    assert_eq!(repository.read("other.rs")?, source);
    assert!(repository.status()?.contains(" D example.rs"));
    assert!(repository.status()?.contains(" M other.rs"));

    Ok(())
}

#[test]
fn untracked_file_prevents_deletion_staging() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::from_scenario("deletion-with-untracked")?;
    let index = repository.index("example.rs")?;
    let source = repository.read("untracked.rs")?;

    repository.run_real("fix")?;

    assert_eq!(repository.index("example.rs")?, index);
    assert_eq!(repository.read("untracked.rs")?, source);
    assert!(repository.status()?.contains(" D example.rs"));
    assert!(repository.status()?.contains("?? untracked.rs"));

    Ok(())
}

#[test]
fn developer_configuration_is_preserved_and_excluded() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::from_scenario("configuration-developer")?;
    let path = ".standardizer/dev.json";
    let source = repository.read(path)?;

    repository.run_real("fix")?;

    assert_eq!(repository.read(path)?, source);
    assert!(!repository.status()?.contains(path));

    Ok(())
}

#[test]
fn tracked_developer_configuration_is_untracked() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::from_scenario("configuration-tracked-developer")?;
    let path = ".standardizer/dev.json";
    let source = repository.read(path)?;

    repository.run_real("fix")?;

    assert_eq!(repository.read(path)?, source);
    assert!(repository.git(&["ls-files", "--", path])?.is_empty());

    Ok(())
}

#[test]
fn invalid_configuration_prevents_formatting() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::from_scenario("configuration-invalid")?;
    let source = repository.read("example.rs")?;
    let index = repository.index("example.rs")?;

    assert!(repository.run_real("fix").is_err());
    assert_eq!(repository.read("example.rs")?, source);
    assert_eq!(repository.index("example.rs")?, index);

    Ok(())
}
