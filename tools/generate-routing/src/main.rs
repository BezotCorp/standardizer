mod routing;

use std::process::ExitCode;

fn main() -> ExitCode {
    match routing::generate() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("generate-routing: {error}");
            ExitCode::FAILURE
        }
    }
}
