mod conflict;
mod git;
mod invalid;
mod partial;
mod repository;
mod tracked;
mod unstaged;
mod untracked;

use std::{env, error::Error, process::ExitCode};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("prepare-test-repository: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut arguments = env::args_os().skip(1);

    let scenario = arguments
        .next()
        .ok_or("usage: prepare-test-repository <scenario>")?;

    if arguments.next().is_some() {
        return Err("unexpected additional argument".into());
    }

    let prepare = match scenario.to_str() {
        Some("untracked") => untracked::prepare,
        Some("tracked") => tracked::prepare,
        Some("unstaged") => unstaged::prepare,
        Some("partial") => partial::prepare,
        Some("conflict") => conflict::prepare,
        Some("invalid") => invalid::prepare,
        _ => return Err("unknown preparation scenario".into()),
    };

    let path = repository::prepare(prepare)?;
    println!("{}", path.display());
    Ok(())
}
