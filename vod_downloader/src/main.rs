use std::process::ExitCode;

#[macro_use]
mod logging;
mod cli;
mod downloader;

fn main() -> ExitCode {
    // Default logging level before parsing arguments
    {
        logging::add_sink(Box::new(logging::ConsoleSink::new(None)));
    }
    
    // Parse arguments and get the download options
    let arguments = cli::parse_cli_arguments(std::env::args().collect());
    let options = cli::parse_cli_options(arguments);
    
    let Ok(options) = options else {
        error!("{}", options.unwrap_err());
        return ExitCode::FAILURE;
    };
    
    trace!("Options: {:?}", options);
    
    // Set logging based on download options
    {
        logging::clear_sinks();

        let log_level: logging::LogLevel = if options.verbose {
            logging::LogLevel::Trace
        } else {
            logging::LogLevel::Info
        };
        
        logging::add_sink(Box::new(logging::ConsoleSink::new(Some(log_level.clone()))));

        if let Some(ref path) = options.log_file {
            let filesink = logging::FileSink::new(path.as_path(), Some(log_level));

            let Ok(filesink) = filesink else {
                error!("{}", filesink.unwrap_err().kind());
                return ExitCode::FAILURE;
            };

            logging::add_sink(Box::new(filesink));
        }
    }

    // Retrieve all episodes to download
    let episodes_m3u_id = downloader::retrieve_episodes(&options);

    let Ok((episodes, m3u_id)) = episodes_m3u_id else {
        error!("{}", episodes_m3u_id.unwrap_err());
        return ExitCode::FAILURE;
    };

    trace!("Episodes: {:?}", episodes);

    // Download all episodes
    for (_season_num, season) in episodes
    {
        for episode in season.episodes
        {
            let result = downloader::download_episode(&options, &episode, m3u_id);

            if let Err(error) = result {
                error!("{}", error);
            }
        }
    }

    ExitCode::SUCCESS
}
