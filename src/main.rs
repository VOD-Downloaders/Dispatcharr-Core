use std::process::ExitCode;
use chrono;

#[macro_use]
mod logging;
mod cli;
mod recipe;
mod downloader;

fn main() -> ExitCode {
    // Default logging level before parsing arguments
    {
        logging::add_sink(Box::new(logging::ConsoleSink::new(None)));
    }
    
    // Parse arguments and get the download options
    info!("Parsing cli arguments...");

    let arguments = cli::parse_cli_arguments(std::env::args().collect());
    let options = cli::parse_cli_options(arguments);
    
    let Ok(options) = options else {
        error!("{}", options.unwrap_err());
        return ExitCode::FAILURE;
    };
    
    // Set logging based on download options/cli parameters
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
                error!("Failed to open log file '{}' with error: {}.", path.display(), filesink.unwrap_err().kind());
                return ExitCode::FAILURE;
            };
            
            logging::add_sink(Box::new(filesink));
        }
    }

    trace!("Options: {:?}", options);

    // Retrieve all episodes available
    info!("Retrieving episodes from Dispatcharr using HTTP GET.");

    let episodes_m3u_id = downloader::retrieve_episodes(&options);

    let Ok((retrieved_episodes, m3u_id)) = episodes_m3u_id else {
        error!("{}", episodes_m3u_id.unwrap_err());
        return ExitCode::FAILURE;
    };

    trace!("Episodes available: {:?}", retrieved_episodes);

    // Setup stats
    let mut total: u32 = 0;
    if options.recipe.seasons.is_empty() {
        total = retrieved_episodes.values().map(|s| s.episodes.len()).sum::<usize>() as u32;
    }
    else
    {
        for (season_num, season) in &options.recipe.seasons
        {
            if season.episodes.len() == 0 {
                match retrieved_episodes.get(season_num)
                {
                    Some(retrieved_season) => {
                        total += retrieved_season.episodes.len() as u32;
                    },
                    None => {
                        warning!("Recipe wants season {0}, but no season {0} in HTTP response.", season_num);
                    }
                }
            }
            else { 
                total += season.episodes.len() as u32; 
            }
        }
    }

    let mut fails: u32 = 0;
    let mut succeeded: u32 = 0;

    // Download all episodes
    for (season_num, season) in retrieved_episodes
    {
        // Season not requested
        if !options.recipe.seasons.is_empty() && (!options.recipe.seasons.contains_key(&season_num) || options.recipe.exclude.contains(&season_num)) {
            continue;
        }

        for episode in season.episodes
        {
            // Episode not requested
            if !options.recipe.seasons.is_empty() 
            {
                let season = &options.recipe.seasons.get(&season_num).unwrap();
                if !season.episodes.is_empty() && (!season.episodes.contains(&episode.episode_number) || season.exclude.contains(&episode.episode_number)) {
                    continue;
                }
            }

            info!("[{}/{}] Starting download for episode: \"{}\".", succeeded + fails + 1, total, episode.title);

            let begin = chrono::Local::now();

            let result = downloader::download_episode(&options, &episode, m3u_id);

            if let Err(error) = result {
                error!("{}", error);
                fails += 1;
            }
            else {
                let end = chrono::Local::now();
                let time = end - begin;

                let hours = time.num_hours();
                let minutes = (time - chrono::TimeDelta::hours(hours)).num_minutes();
                let seconds = (time - chrono::TimeDelta::minutes(minutes)).num_seconds();

                info!("Download for \"{}\" finished in {} hours, {} minutes and {} seconds.", episode.title, hours, minutes, seconds);
                succeeded += 1;
            }
        }
    }

    // Final exit message
    if total == succeeded {
        info!("Downloads for all {} episodes succeeded!", total);
        ExitCode::SUCCESS
    } else if succeeded > 0 {
        warning!("Only {} out of {} episodes were successfully downloaded.", succeeded, total);
        ExitCode::FAILURE
    } else {
        error!("All downloads failed to complete...");
        ExitCode::FAILURE
    }
}