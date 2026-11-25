mod cargo_metadata;
mod git_repository;
mod goals;
mod resolved_tool;
mod routing;
mod runnung_operation;
mod tools;
mod tools_gui;
mod tracked_file;

use goals::ConformancePipeline;
use std::{env, error::Error, iter, process::ExitCode};
use tools::ToolCommand;

const HELP: &str = concat!(
    "standardizer ",
    env!("CARGO_PKG_VERSION"),
    "\n",
    env!("CARGO_PKG_DESCRIPTION"),
    "

Usage: standardizer <COMMAND>

Commands:
  check      Report non-conforming files without modifying them
  fix        Apply supported corrections and stage the files that changed
  tools      Manage formatting tools (run 'standardizer tools --help')
  help       Print this help

Options:
  -h, --help     Print this help
  -V, --version  Print version
"
);

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
    let Some(command) = arguments.next() else {
        eprint!("{HELP}");

        return Err("missing command".into());
    };

    match command.to_str() {
        Some("-V" | "--version") => {
            println!("standardizer {}", env!("CARGO_PKG_VERSION"));

            Ok(())
        }

        Some("-h" | "--help" | "help") => {
            print!("{HELP}");

            Ok(())
        }

        Some("tools") => ToolCommand::run(arguments),
        Some(name @ ("check" | "fix")) => {
            let rest: Vec<_> = arguments.collect();

            if rest
                .iter()
                .any(|argument| matches!(argument.to_str(), Some("-h" | "--help")))
            {
                print!("{}", conformance_help(name));

                return Ok(());
            }

            ConformancePipeline::run(iter::once(command.clone()).chain(rest))
        }

        _ => Err(format!(
            "unknown command '{}'\n\nRun 'standardizer --help' for usage.",
            command.to_string_lossy(),
        )
        .into()),
    }
}

fn conformance_help(name: &str) -> String {
    let description = if name == "check" {
        "Report files tracked by Git that do not conform, without modifying them.\n\
         Exits with a non-zero status when a file does not conform."
    } else {
        "Apply supported corrections to files tracked by Git and stage only\n\
         the files that were changed."
    };

    format!(
        "{description}\n\n\
         Usage: standardizer {name}\n\n\
         Rust files are formatted with rustfmt; JavaScript, TypeScript, JSON,\n\
         CSS and GraphQL files are formatted with Biome.\n\n\
         Options:\n  -h, --help  Print this help\n"
    )
}
