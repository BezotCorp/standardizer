use crate::{
    biome_installation::BiomeInstallation, tool_manager::ToolManager, tools_gui::ToolsGui,
};
use standardizer_common::tools::Tool;
use std::{env, error::Error, ffi::OsString};

pub(crate) struct ToolCommand;

impl ToolCommand {
    pub(crate) fn run(mut arguments: impl Iterator<Item = OsString>) -> Result<(), Box<dyn Error>> {
        let command = arguments.next().and_then(|value| value.into_string().ok());

        match command.as_deref() {
            None | Some("ui") => {
                Self::require_no_more(arguments)?;

                ToolsGui::run(env::current_dir().ok())
            }

            Some("status") => {
                Self::require_no_more(arguments)?;
                Self::status()
            }

            Some("install") => {
                let tool = Self::tool(&mut arguments)?;

                Self::require_no_more(arguments)?;

                let path = ToolManager::install(tool)?;

                println!(
                    "{} {} installed at {}",
                    tool.display_name(),
                    Self::managed_version()?,
                    path.display(),
                );

                Ok(())
            }

            Some("uninstall") => {
                let tool = Self::tool(&mut arguments)?;

                Self::require_no_more(arguments)?;

                if ToolManager::uninstall(tool)? {
                    println!("{} managed installation removed", tool.display_name(),);
                } else {
                    println!(
                        "{} has no Standardizer-managed installation",
                        tool.display_name(),
                    );
                }

                Ok(())
            }

            _ => Err(Self::usage().into()),
        }
    }

    fn status() -> Result<(), Box<dyn Error>> {
        let root = env::current_dir().ok();

        for (index, tool) in ToolManager::MANAGED_TOOLS.iter().copied().enumerate() {
            if index > 0 {
                println!();
            }

            let status = ToolManager::status(tool, root.as_deref())?;

            println!(
                "{} ({})",
                status.tool().display_name(),
                status.tool().name(),
            );

            match status.active() {
                Some(active) => {
                    println!("  status: available");

                    println!("  source: {}", active.source().label(),);

                    println!("  path: {}", active.path().display(),);

                    println!("  version: {}", active.version(),);
                }

                None => {
                    println!("  status: unavailable");
                }
            }

            println!(
                "  managed: {}",
                if status.managed_installed() {
                    "installed"
                } else {
                    "not installed"
                },
            );

            println!("  managed path: {}", status.managed_path().display(),);
        }

        Ok(())
    }

    fn managed_version() -> Result<semver::Version, Box<dyn Error>> {
        BiomeInstallation::managed_version()
    }

    fn tool(arguments: &mut impl Iterator<Item = OsString>) -> Result<Tool, Box<dyn Error>> {
        let name = arguments
            .next()
            .and_then(|value| value.into_string().ok())
            .ok_or_else(Self::usage)?;

        ToolManager::from_name(&name).ok_or_else(|| format!("unknown tool: {name}",).into())
    }

    fn require_no_more(
        mut arguments: impl Iterator<Item = OsString>,
    ) -> Result<(), Box<dyn Error>> {
        if arguments.next().is_some() {
            return Err(Self::usage().into());
        }

        Ok(())
    }

    fn usage() -> &'static str {
        "usage: standardizer tools [ui|status|install biome|uninstall biome]"
    }
}
