use std::process::ExitCode;
use std::fs::OpenOptions;

#[macro_use]
mod logging;
mod cli;
mod downloader;

fn main() -> ExitCode {
    logging::add_sink(Box::new(logging::ConsoleSink::new(Some(logging::LogLevel::Trace))));

    let arguments = cli::parse_cli_arguments(std::env::args().collect());
    let options = cli::parse_cli_options(arguments);

    let Ok(options) = options else {
        error!("{}", options.unwrap_err());
        return ExitCode::FAILURE;
    };

    trace!("Options: {:?}", options);

    let episodes_m3u_id = downloader::retrieve_episodes(&options);

    let Ok((episodes, m3u_id)) = episodes_m3u_id else {
        error!("{}", episodes_m3u_id.unwrap_err());
        return ExitCode::FAILURE;
    };

    trace!("Episodes: {:?}", episodes);

    trace!("Opening log file, {}.", options.log_file.display());

    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(options.log_file.as_path());

    let Ok(mut log_file) = log_file else {
        error!("Failed to open log file with error: {}.", log_file.unwrap_err().kind());
        return ExitCode::FAILURE;
    };

    for (_season_num, season) in episodes
    {
        for episode in season.episodes
        {
            let result = downloader::download_episode(&options, &episode, m3u_id, &mut log_file);

            if let Err(error) = result {
                error!("{}", error);
            }
        }
    }

    ExitCode::SUCCESS
}
