use crate::{
    goals::ConformanceIntent,
    routing,
    tools::{Biome, Rustfmt},
    tracked_file::TrackedFile,
};
use standardizer_common::tools::Tool;
use std::{collections::BTreeMap, error::Error, path::Path};

pub(crate) struct Conformance;

impl Conformance {
    pub(crate) fn execute(
        root: &Path,
        files: &[TrackedFile],
        intent: ConformanceIntent,
    ) -> Result<(), Box<dyn Error>> {
        let routing = routing::load()?;
        let mut batches: BTreeMap<Tool, Vec<&Path>> = BTreeMap::new();

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
            let result = match tool {
                Tool::Biome => Biome::execute(root, &paths, intent)?,
                Tool::Rustfmt => Rustfmt::execute(root, &paths, intent)?,
            };
            passed &= result;
        }

        if !passed {
            return Err("conformance failed".into());
        }

        Ok(())
    }
}
