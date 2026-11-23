use bincode_next::Decode;
#[cfg(all(not(feature = "routing-source"), feature = "embedded-routing"))]
use bincode_next::{config, decode_from_slice};
use standardizer_common::{routes::Route, tools::Tool};
#[cfg(feature = "routing-source")]
use std::fs;
use std::{collections::BTreeSet, error::Error, path::Path};

#[cfg(not(any(feature = "routing-source", feature = "embedded-routing")))]
compile_error!("enable at least one routing mode: routing-source or embedded-routing");

#[derive(Decode)]
#[cfg_attr(feature = "routing-source", derive(serde::Deserialize))]
pub(crate) struct Routing {
    routes: Vec<Route>,
}

impl Routing {
    #[cfg(feature = "routing-source")]
    pub(crate) fn load() -> Result<Self, Box<dyn Error>> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/routing.ron");
        let routing: Self = ron::from_str(&fs::read_to_string(path)?)?;

        routing.validate()?;
        Ok(routing)
    }

    #[cfg(all(not(feature = "routing-source"), feature = "embedded-routing"))]
    pub(crate) fn load() -> Result<Self, Box<dyn Error>> {
        let bytes = include_bytes!("../data/routing.bin");
        let (routing, consumed): (Self, usize) = decode_from_slice(bytes, config::standard())?;
        if consumed != bytes.len() {
            return Err("trailing data in routing binary".into());
        }

        routing.validate()?;
        Ok(routing)
    }

    pub(crate) fn validate(&self) -> Result<(), Box<dyn Error>> {
        for route in &self.routes {
            if route.extensions().is_empty() || route.tools().is_empty() {
                return Err("routing rules require extensions and tools".into());
            }
        }

        Ok(())
    }

    pub(crate) fn tools(&self, path: &Path) -> BTreeSet<Tool> {
        let mut selected = BTreeSet::new();

        let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
            return selected;
        };

        for route in &self.routes {
            if route
                .extensions()
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(extension))
            {
                selected.extend(route.tools().iter().copied());
            }
        }

        selected
    }
}
