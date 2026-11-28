use crate::{
    git_repository::GitRepository,
    goals::ConformanceIntent,
    routing,
    tools::{Biome, Rustfmt},
    tracked_file::TrackedFile,
};
use standardizer_common::tools::Tool;
use std::{
    collections::BTreeMap,
    error::Error,
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
};

pub(crate) struct Conformance;

impl Conformance {
    pub(crate) fn execute(
        repository: &GitRepository,
        files: &[TrackedFile],
        intent: ConformanceIntent,
    ) -> Result<BTreeMap<PathBuf, OsString>, Box<dyn Error>> {
        let root = repository.root();
        let routing = routing::load()?;
        let mut batches: BTreeMap<Tool, Vec<&Path>> = BTreeMap::new();

        let tracked: BTreeMap<&Path, &TrackedFile> =
            files.iter().map(|file| (file.path(), file)).collect();

        let mut expected: BTreeMap<PathBuf, OsString> = files
            .iter()
            .map(|file| (file.path().to_path_buf(), file.blob().clone()))
            .collect();

        for file in files {
            for tool in routing.tools(file.path()) {
                batches.entry(tool).or_default().push(file.path());
            }
        }

        if batches.contains_key(&Tool::Rustfmt) && !Rustfmt::verify(root)? {
            batches.remove(&Tool::Rustfmt);
        }

        if batches.contains_key(&Tool::Biome) && !Biome::verify(root)? {
            batches.remove(&Tool::Biome);
        }

        let mut passed = true;

        for (tool, paths) in batches {
            let execute = |command: &mut Command, paths: &[&Path]| {
                Self::execute_checked(repository, &tracked, &mut expected, command, paths, intent)
            };

            let result = match tool {
                Tool::Biome => Biome::execute(root, &paths, intent, execute)?,
                Tool::Rustfmt => Rustfmt::execute(root, &paths, intent, execute)?,
            };

            passed &= result;
        }

        if !passed {
            return Err("conformance failed".into());
        }

        Ok(expected)
    }

    fn execute_checked(
        repository: &GitRepository,
        tracked: &BTreeMap<&Path, &TrackedFile>,
        expected: &mut BTreeMap<PathBuf, OsString>,
        command: &mut Command,
        paths: &[&Path],
        intent: ConformanceIntent,
    ) -> Result<bool, Box<dyn Error>> {
        let mut eligible = Vec::new();

        for &path in paths {
            let Some(blob) = expected.get(path) else {
                continue;
            };

            let file = tracked.get(path).ok_or("missing tracked file")?;

            if repository.matches(file, blob)? {
                eligible.push(path);
            } else {
                expected.remove(path);
                eprintln!("standardizer: skipping changed file {}", path.display());
            }
        }

        if eligible.is_empty() {
            return Ok(true);
        }

        let passed = command
            .args(eligible.iter().map(|path| repository.root().join(path)))
            .status()?
            .success();

        if !passed {
            return Ok(false);
        }

        if intent == ConformanceIntent::FixConformance {
            for path in eligible {
                let file = tracked.get(path).ok_or("missing tracked file")?;

                match repository.capture(file)? {
                    Some(blob) => {
                        expected.insert(path.to_path_buf(), blob);
                    }
                    None => {
                        expected.remove(path);
                        eprintln!("standardizer: skipping changed file {}", path.display(),);
                    }
                }
            }
        }

        Ok(true)
    }
}
