use std::fmt;
use std::fs::File;
use reqwest::blocking::Client;

use super::types::*;
use super::super::cli::DownloadOptions;

/////////////////////////////////////////////////////
// DownloadError
/////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub enum DownloadError
{
    StartDownloadFailed{ title: String, error_type: String },
    DownloadFailed{ title: String, exit_code: i32 },
    FailedToCreateFile{ title: String, file: String,  error_type: String },
    FailedToCopyContentsToFile{ title: String, file: String,  error_type: String }
}

impl fmt::Display for DownloadError
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result 
    {
        match self
        {
            DownloadError::StartDownloadFailed{ title, error_type } => { write!(formatter, "Starting download: \"{}\" failed with error: {}.", title, error_type) },
            DownloadError::DownloadFailed{ title, exit_code } => { write!(formatter, "Download: \"{}\" exited with exit code: {} and subsequently failed.", title, exit_code) },
            DownloadError::FailedToCreateFile{ title, file, error_type } => { write!(formatter, "Download: \"{}\" failed, because of being unable to create file \"{}\" due to error: {}.", title, file, error_type) },
            DownloadError::FailedToCopyContentsToFile{ title, file, error_type } => { write!(formatter, "Download: \"{}\" failed, because of being unable to copy HTTP response contents to file \"{}\" with errorcode: {}.", title, file, error_type) },
        }
    }   
}

/////////////////////////////////////////////////////
// Downloader
/////////////////////////////////////////////////////
pub fn download_episode(options: &DownloadOptions, episode: &Episode, m3u_id: M3UID) -> Result<(), DownloadError>
{
    let url = format!("{}/proxy/vod/episode/{}?m3u_account_id={}", options.url, episode.uuid, m3u_id);
    let output_file = format!("{}.{}", episode.title.chars().filter(|c| !c.is_whitespace()).collect::<String>(), episode.container_extension);

    let mut last_error: DownloadError = DownloadError::StartDownloadFailed { title: "".to_string(), error_type: "".to_string() }; // Must be initialized
    for attempt in 1..=options.max_reties 
    {
        match download_attempt(&url, &output_file, episode.title.as_str()) 
        {
            Ok(()) => return Ok(()),
            Err(e) => 
            {
                warning!("[Attempt {}/{}] Failed with error: {}.", attempt, options.max_reties, e);
                last_error = e;
            }
        }
    }

    Err(last_error)
}

fn download_attempt(url: &str, output_file: &str, debug_title: &str) -> Result<(), DownloadError>
{
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .map_err(|e| DownloadError::StartDownloadFailed { title: debug_title.to_string(), error_type: e.to_string() })?;

    let mut response = client
        .get(url)
        .send()
        .map_err(|e| DownloadError::StartDownloadFailed { title: debug_title.to_string(), error_type: e.to_string() })?;

    if !response.status().is_success() {
        return Err(DownloadError::DownloadFailed { title: debug_title.to_string(), exit_code: response.status().as_u16() as i32 });
    }

    let mut file = File::create(output_file)
        .map_err(|e| DownloadError::FailedToCreateFile { title: debug_title.to_string(), file: output_file.to_string(), error_type: e.to_string() })?;

    response.copy_to(&mut file)
        .map_err(|e| DownloadError::FailedToCopyContentsToFile { title: debug_title.to_string(), file: output_file.to_string(), error_type: e.to_string() })?;

    Ok(())
}