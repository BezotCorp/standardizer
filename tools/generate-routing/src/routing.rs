use bincode_next::{Encode, config, encode_to_vec};
use common::route::Route;
use serde::Deserialize;
use std::{error::Error, fs, path::Path};

#[derive(Deserialize, Encode)]
struct Routing {
    routes: Vec<Route>,
}

impl Routing {
    fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self
            .routes
            .iter()
            .any(|route| route.extensions().is_empty() || route.tools().is_empty())
        {
            return Err("routing rules require extensions and tools".into());
        }

        Ok(())
    }
}

pub(crate) fn generate() -> Result<(), Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .ok_or("generate-routing must be located under tools/generate-routing")?;
    let source = fs::read_to_string(root.join("data/standardizer/routing.ron"))?;
    let routing: Routing = ron::from_str(&source)?;

    routing.validate()?;
    let encoded = encode_to_vec(&routing, config::standard())?;
    fs::write(root.join("data/standardizer/routing.bin"), encoded)?;
    Ok(())
}
