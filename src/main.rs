mod biome;
mod biome_installation;
mod cargo_metadata;
mod conformance;
mod conformance_intent;
mod conformance_pipeline;
mod git_repository;
mod resolved_tool;
mod routing;
mod rustfmt;
mod tool_command;
mod tool_id;
mod tool_manager;
mod tool_source;
mod tool_status;
mod tools_gui;
mod tracked_file;

use conformance_pipeline::ConformancePipeline;
use std::{env, error::Error, process::ExitCode};
use tool_command::ToolCommand;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("standardizer: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut arguments = env::args_os().skip(1);

    match arguments.next() {
        Some(command) if command.to_str() == Some("tools") => {
            ToolCommand::run(arguments)
        }
        Some(command) => {
            ConformancePipeline::run(
                std::iter::once(command).chain(arguments),
            )
        }
        None => ConformancePipeline::run(std::iter::empty()),
    }
}
