use crate::{
    biome::Biome, conformance_intent::ConformanceIntent, routing::Routing, rustfmt::Rustfmt,
    tracked_file::TrackedFile,
};
use common::tool::Tool;
use std::{collections::BTreeMap, error::Error, path::Path};

pub(crate) struct Conformance;

impl Conformance {
    pub(crate) fn execute(
        root: &Path,
        files: &[TrackedFile],
        intent: ConformanceIntent,
    ) -> Result<(), Box<dyn Error>> {
        let routing = Routing::load()?;
        let mut batches: BTreeMap<Tool, Vec<&Path>> = BTreeMap::new();

        for file in files {
            for tool in routing.tools(file.path()) {
                batches.entry(tool).or_default().push(file.path());
            }
        }

        if batches.contains_key(&Tool::Rustfmt) {
            Rustfmt::verify(root)?;
        }
        if batches.contains_key(&Tool::Biome) {
            Biome::verify(root)?;
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
