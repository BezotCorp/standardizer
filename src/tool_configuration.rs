use crate::tool_id::ToolId;
use semver::Version;
use serde::Deserialize;
use std::{
    collections::BTreeSet,
    error::Error,
};

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ToolDefinition {
    id: ToolId,
    minimum_version: String,
    managed_version: String,
}

impl ToolDefinition {
    pub(crate) fn id(&self) -> ToolId {
        self.id
    }

    pub(crate) fn minimum_version(
        &self,
    ) -> Result<Version, Box<dyn Error>> {
        Ok(Version::parse(
            &self.minimum_version,
        )?)
    }

    pub(crate) fn managed_version(
        &self,
    ) -> Result<Version, Box<dyn Error>> {
        Ok(Version::parse(
            &self.managed_version,
        )?)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ToolConfiguration {
    tools: Vec<ToolDefinition>,
}

impl ToolConfiguration {
    pub(crate) fn load(
    ) -> Result<Self, Box<dyn Error>> {
        let configuration: Self = ron::from_str(
            include_str!("../data/tools.ron"),
        )?;

        configuration.validate()?;

        Ok(configuration)
    }

    pub(crate) fn definition(
        &self,
        id: ToolId,
    ) -> Result<&ToolDefinition, Box<dyn Error>> {
        self.tools
            .iter()
            .find(|tool| tool.id() == id)
            .ok_or_else(|| {
                format!(
                    "missing tool definition for {}",
                    id.name(),
                )
                .into()
            })
    }

    fn validate(
        &self,
    ) -> Result<(), Box<dyn Error>> {
        let mut ids = BTreeSet::new();

        for tool in &self.tools {
            if !ids.insert(tool.id()) {
                return Err(
                    format!(
                        "duplicate tool definition for {}",
                        tool.id().name(),
                    )
                    .into(),
                );
            }

            let minimum =
                tool.minimum_version()?;

            let managed =
                tool.managed_version()?;

            if managed < minimum {
                return Err(
                    format!(
                        "{} managed version {} is older than minimum supported version {}",
                        tool.id().name(),
                        managed,
                        minimum,
                    )
                    .into(),
                );
            }
        }

        for tool in ToolId::ALL {
            if !ids.contains(&tool) {
                return Err(
                    format!(
                        "missing tool definition for {}",
                        tool.name(),
                    )
                    .into(),
                );
            }
        }

        Ok(())
    }
}
