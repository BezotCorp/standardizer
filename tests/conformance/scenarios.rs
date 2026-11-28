use crate::support::TestRepository;
use std::error::Error;

#[test]
fn unchanged_tracked_file_is_formatted_and_staged() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;
    repository.tracked("a.rs", "fn a() {}\n")?;

    repository.fix("normal")?;

    assert_eq!(repository.index("a.rs")?, "fn a() {}\n// formatted\n",);
    assert_eq!(repository.status()?, "M  a.rs\n");

    Ok(())
}

#[test]
fn fully_staged_file_is_formatted_and_staged() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;
    repository.tracked("a.rs", "fn a() {}\n")?;

    repository.write("a.rs", "fn changed() {}\n")?;
    repository.git(&["add", "--", "a.rs"])?;

    repository.fix("normal")?;

    assert_eq!(repository.index("a.rs")?, "fn changed() {}\n// formatted\n",);
    assert_eq!(repository.status()?, "M  a.rs\n");

    Ok(())
}

#[test]
fn unstaged_file_is_preserved() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;
    repository.tracked("a.rs", "fn a() {}\n")?;

    repository.write("a.rs", "fn changed() {}\n")?;

    repository.fix("normal")?;

    assert_eq!(repository.index("a.rs")?, "fn a() {}\n");
    assert_eq!(repository.read("a.rs")?, "fn changed() {}\n");
    assert_eq!(repository.status()?, " M a.rs\n");

    Ok(())
}

#[test]
fn partially_staged_file_is_preserved() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;
    repository.tracked("a.rs", "fn a() {}\n")?;

    repository.write("a.rs", "fn staged() {}\n")?;
    repository.git(&["add", "--", "a.rs"])?;
    repository.write("a.rs", "fn unstaged() {}\n")?;

    repository.fix("normal")?;

    assert_eq!(repository.index("a.rs")?, "fn staged() {}\n");
    assert_eq!(repository.read("a.rs")?, "fn unstaged() {}\n");
    assert_eq!(repository.status()?, "MM a.rs\n");

    Ok(())
}

#[test]
fn untracked_file_is_preserved() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;
    repository.write("a.rs", "fn a() {}\n")?;

    repository.fix("normal")?;

    assert_eq!(repository.read("a.rs")?, "fn a() {}\n");
    assert_eq!(repository.status()?, "?? a.rs\n");

    Ok(())
}

#[test]
fn external_change_before_formatting_is_preserved() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;
    repository.tracked("a.rs", "fn a() {}\n")?;
    repository.tracked("b.rs", "fn b() {}\n")?;

    repository.fix("change_before")?;

    assert_eq!(repository.index("a.rs")?, "fn a() {}\n// formatted\n",);
    assert_eq!(repository.index("b.rs")?, "fn b() {}\n");
    assert_eq!(repository.read("b.rs")?, "fn external() {}\n");
    assert_eq!(repository.status()?, "M  a.rs\n M b.rs\n");

    Ok(())
}

#[test]
fn external_change_after_formatting_is_not_staged() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;
    repository.tracked("a.rs", "fn a() {}\n")?;
    repository.tracked("b.rs", "fn b() {}\n")?;

    repository.fix("change_after")?;

    assert_eq!(repository.index("a.rs")?, "fn a() {}\n");
    assert_eq!(repository.read("a.rs")?, "fn external() {}\n");
    assert_eq!(repository.index("b.rs")?, "fn b() {}\n// formatted\n",);
    assert_eq!(repository.status()?, " M a.rs\nM  b.rs\n");

    Ok(())
}

#[test]
fn deletion_before_formatting_is_not_staged() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;
    repository.write(".standardizer/dev.json", r#"{"deletion":"ignore"}"#)?;
    repository.tracked("a.rs", "fn a() {}\n")?;
    repository.tracked("b.rs", "fn b() {}\n")?;

    repository.fix("delete_before")?;

    assert_eq!(repository.index("b.rs")?, "fn b() {}\n");
    assert!(!repository.exists("b.rs"));
    assert_eq!(repository.status()?, "M  a.rs\n D b.rs\n");

    Ok(())
}

#[test]
fn deletion_after_formatting_is_not_staged() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;
    repository.write(".standardizer/dev.json", r#"{"deletion":"ignore"}"#)?;
    repository.tracked("a.rs", "fn a() {}\n")?;
    repository.tracked("b.rs", "fn b() {}\n")?;

    repository.fix("delete_after")?;

    assert_eq!(repository.index("a.rs")?, "fn a() {}\n");
    assert!(!repository.exists("a.rs"));
    assert_eq!(repository.status()?, " D a.rs\nM  b.rs\n");

    Ok(())
}

#[test]
fn external_change_to_unrouted_file_is_not_staged() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;
    repository.tracked("a.rs", "fn a() {}\n")?;
    repository.tracked("notes.txt", "original\n")?;

    repository.fix("change_unrouted")?;

    assert_eq!(repository.index("notes.txt")?, "original\n");
    assert_eq!(repository.read("notes.txt")?, "external\n");
    assert_eq!(repository.status()?, "M  a.rs\n M notes.txt\n");

    Ok(())
}

#[test]
fn external_index_change_is_preserved() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;
    repository.tracked("a.rs", "fn a() {}\n")?;
    repository.tracked("b.rs", "fn b() {}\n")?;

    repository.fix("change_index")?;

    assert_eq!(repository.index("a.rs")?, "fn external() {}\n");
    assert_eq!(repository.read("a.rs")?, "fn external() {}\n");
    assert_eq!(repository.index("b.rs")?, "fn b() {}\n// formatted\n",);

    Ok(())
}

#[cfg(unix)]
#[test]
fn executable_mode_change_is_preserved() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;
    repository.tracked("a.rs", "fn a() {}\n")?;
    repository.tracked("b.rs", "fn b() {}\n")?;

    repository.fix("change_mode")?;

    assert_eq!(repository.index("b.rs")?, "fn b() {}\n");
    assert_eq!(repository.read("b.rs")?, "fn b() {}\n");
    assert_eq!(repository.status()?, "M  a.rs\n M b.rs\n");

    Ok(())
}

#[test]
fn check_does_not_modify_files_or_index() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;
    repository.tracked("a.rs", "fn a() {}\n")?;

    repository.check()?;

    assert_eq!(repository.index("a.rs")?, "fn a() {}\n");
    assert_eq!(repository.read("a.rs")?, "fn a() {}\n");
    assert_eq!(repository.status()?, "");

    Ok(())
}

#[test]
fn failed_formatter_does_not_stage_changes() -> Result<(), Box<dyn Error>> {
    let repository = TestRepository::new()?;
    repository.tracked("a.rs", "fn a() {}\n")?;

    assert!(repository.fix("fail").is_err());

    assert_eq!(repository.index("a.rs")?, "fn a() {}\n");
    assert_eq!(repository.read("a.rs")?, "fn a() {}\n// formatted\n",);
    assert_eq!(repository.status()?, " M a.rs\n");

    Ok(())
}

#[test]
fn only_unstaged_deletions_are_automatically_staged() -> Result<(), Box<dyn std::error::Error>> {
    let repository = TestRepository::new()?;

    repository.tracked("a.rs", "fn a() {}\n")?;
    repository.tracked("b.rs", "fn b() {}\n")?;

    repository.git(&["rm", "--", "a.rs"])?;
    repository.git(&["reset", "--", "a.rs"])?;

    repository.fix("normal")?;

    assert_eq!(repository.status()?, "D  a.rs\nM  b.rs\n");

    Ok(())
}

#[test]
fn deletion_before_formatting_is_automatically_staged() -> Result<(), Box<dyn std::error::Error>> {
    let repository = TestRepository::new()?;

    repository.tracked("a.rs", "fn a() {}\n")?;
    repository.tracked("b.rs", "fn b() {}\n")?;

    repository.fix("delete_before")?;

    assert_eq!(repository.status()?, "M  a.rs\nD  b.rs\n");

    Ok(())
}

#[test]
fn deletion_after_formatting_is_automatically_staged() -> Result<(), Box<dyn std::error::Error>> {
    let repository = TestRepository::new()?;

    repository.tracked("a.rs", "fn a() {}\n")?;
    repository.tracked("b.rs", "fn b() {}\n")?;

    repository.fix("delete_after")?;

    assert_eq!(repository.status()?, "D  a.rs\nM  b.rs\n");

    Ok(())
}

#[test]
fn unstaged_modification_prevents_automatic_deletion() -> Result<(), Box<dyn std::error::Error>> {
    let repository = TestRepository::new()?;

    repository.tracked("a.rs", "fn a() {}\n")?;
    repository.tracked("b.rs", "fn b() {}\n")?;
    repository.tracked("notes.txt", "original\n")?;

    repository.write("notes.txt", "modified\n")?;

    repository.fix("delete_before")?;

    assert_eq!(repository.status()?, "M  a.rs\n D b.rs\n M notes.txt\n",);

    Ok(())
}

#[test]
fn untracked_file_prevents_automatic_deletion() -> Result<(), Box<dyn std::error::Error>> {
    let repository = TestRepository::new()?;

    repository.tracked("a.rs", "fn a() {}\n")?;
    repository.tracked("b.rs", "fn b() {}\n")?;

    repository.write("new.txt", "new\n")?;

    repository.fix("delete_before")?;

    assert_eq!(repository.status()?, "M  a.rs\n D b.rs\n?? new.txt\n",);

    Ok(())
}

#[test]
fn staged_deletion_is_preserved() -> Result<(), Box<dyn std::error::Error>> {
    let repository = TestRepository::new()?;

    repository.tracked("a.rs", "fn a() {}\n")?;
    repository.tracked("b.rs", "fn b() {}\n")?;

    repository.git(&["rm", "--", "a.rs"])?;
    repository.write("new.txt", "new\n")?;

    repository.fix("normal")?;

    assert_eq!(repository.status()?, "D  a.rs\nM  b.rs\n?? new.txt\n",);

    Ok(())
}

#[test]
fn developer_configuration_is_automatically_ignored() -> Result<(), Box<dyn std::error::Error>> {
    let repository = TestRepository::new()?;

    repository.tracked("a.rs", "fn a() {}\n")?;
    repository.write(".standardizer/dev.json", "{}")?;

    repository.fix("normal")?;

    assert_eq!(repository.status()?, "M  a.rs\n");
    assert_eq!(repository.read(".standardizer/dev.json")?, "{}");

    Ok(())
}

#[test]
fn tracked_developer_configuration_is_removed_from_index() -> Result<(), Box<dyn std::error::Error>>
{
    let repository = TestRepository::new()?;

    repository.tracked("a.rs", "fn a() {}\n")?;
    repository.tracked(".standardizer/dev.json", "{}")?;

    repository.fix("normal")?;

    assert_eq!(repository.status()?, "D  .standardizer/dev.json\nM  a.rs\n",);

    assert_eq!(repository.read(".standardizer/dev.json")?, "{}");

    Ok(())
}

#[test]
fn invalid_configuration_prevents_formatting() -> Result<(), Box<dyn std::error::Error>> {
    let repository = TestRepository::new()?;

    repository.tracked("a.rs", "fn a() {}\n")?;
    repository.write(".standardizer/dev.json", r#"{"deletion":"invalid"}"#)?;

    assert!(repository.fix("normal").is_err());
    assert_eq!(repository.read("a.rs")?, "fn a() {}\n");
    assert_eq!(repository.index("a.rs")?, "fn a() {}\n");

    Ok(())
}

#[test]
fn check_does_not_protect_developer_configuration() -> Result<(), Box<dyn std::error::Error>> {
    let repository = TestRepository::new()?;

    repository.tracked("a.rs", "fn a() {}\n")?;
    repository.write(".standardizer/dev.json", "{}")?;

    repository.check()?;

    assert_eq!(repository.status()?, "?? .standardizer/dev.json\n",);

    Ok(())
}
