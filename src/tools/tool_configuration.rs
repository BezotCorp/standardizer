use semver::Version;
use standardizer_common::tools::ToolConfiguration;
use std::error::Error;

pub(crate) fn load() -> Result<ToolConfiguration, Box<dyn Error>> {
    let bytes = include_bytes!("../../data/tools.bin");

    let configuration = ToolConfiguration::decode(bytes)?;

    for definition in configuration.tools() {
        let minimum = Version::parse(definition.minimum_version())?;

        let managed = Version::parse(definition.managed_version())?;

        if managed < minimum {
            return Err(format!(
                "{} managed version {} is older than minimum supported version {}",
                definition.id().name(),
                managed,
                minimum,
            )
            .into());
        }
    }

    Ok(configuration)
}
