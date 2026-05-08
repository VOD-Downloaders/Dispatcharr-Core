use std::process::ExitCode;
use std::fs::OpenOptions;

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
        eprintln!("Failed to retrieve episodes for series with id: '{}'. \n\tError: {:?}", options.series_id, episodes_m3u_id.unwrap_err());
        return ExitCode::FAILURE;
    };

    println!("Episodes: {:?}", episodes);

    println!("Opening log file, {}.", options.log_file.display());

    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(options.log_file.as_path());

    let Ok(mut log_file) = log_file else {
        eprintln!("Failed to open log file with error: \n\t{:?}", log_file.unwrap_err());
        return ExitCode::FAILURE;
    };

    for (season_num, season) in episodes
    {
        for episode in season.episodes
        {
            let result = downloader::download_episode(&options, &episode, m3u_id, &mut log_file);

            if let Err(error) = result {
                eprintln!("Failed to download Episode {} of season {} with error: {:?}", episode.episode_number, season_num, error);
            }
        }
    }

    ExitCode::SUCCESS
}
