use std::process::ExitCode;

mod cli;
mod downloader;

fn main() -> ExitCode {
    let arguments = cli::parse_cli_arguments(std::env::args().collect());
    let options = cli::parse_cli_options(arguments);

    let Ok(options) = options else {
        eprintln!("Failed to parse cli arguments with error: \n\t{:?}", options.unwrap_err());
        return ExitCode::FAILURE;
    };

    println!("Options: {:?}", options);

    let episodes_m3u_id = downloader::retrieve_episodes(&options);

    let Ok((episodes, m3u_id)) = episodes_m3u_id else {
        eprintln!("Failed to retrieve episodes for series with id: ''. \n\tError: {:?}", episodes_m3u_id.unwrap_err());
        return ExitCode::FAILURE;
    };

    println!("Episodes: {:?}", episodes);
    
    downloader::download_episode(&options, episodes.get(&1).unwrap().episodes.get(0).unwrap(), m3u_id, true);

    ExitCode::FAILURE
}
