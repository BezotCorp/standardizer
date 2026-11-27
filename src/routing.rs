use standardizer_common::routes::Routing;
use std::error::Error;
#[cfg(feature = "routing-source")]
use std::{fs, path::Path};

#[cfg(not(any(feature = "routing-source", feature = "embedded-routing")))]
compile_error!("enable at least one routing mode: routing-source or embedded-routing");

#[cfg(feature = "routing-source")]
pub(crate) fn load() -> Result<Routing, Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/routing.ron");
    Routing::from_ron(&fs::read_to_string(path)?)
}

#[cfg(all(not(feature = "routing-source"), feature = "embedded-routing"))]
pub(crate) fn load() -> Result<Routing, Box<dyn Error>> {
    let bytes = include_bytes!("../data/routing.bin");
    Routing::decode(bytes)
}
