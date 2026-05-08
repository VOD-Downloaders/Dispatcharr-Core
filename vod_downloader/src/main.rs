use std::process::ExitCode;

mod cli;
mod api;

fn main() -> ExitCode {
    let arguments = cli::parse_cli_arguments(std::env::args().collect());
    let options = cli::parse_cli_options(arguments);

    let Ok(options) = options else {
        eprintln!("Failed to parse cli arguments with error: \n\t{:?}", options.unwrap_err());
        return ExitCode::FAILURE;
    };

    println!("Options: {:?}", options);

    ExitCode::FAILURE
}
