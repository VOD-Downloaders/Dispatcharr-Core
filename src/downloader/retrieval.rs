use std::fmt;
use serde::Deserialize;
use std::collections::HashMap;

use super::types::*;
use super::DownloadOptions;

/////////////////////////////////////////////////////
// JSON
/////////////////////////////////////////////////////
#[derive(Deserialize)]
struct ProviderInfoResponse 
{
    m3u_account: M3uAccountResponse,
    episodes: HashMap<String, Vec<EpisodeResponse>>, // Key is season number as a string: "1", "2", ...
}

#[derive(Deserialize)]
struct M3uAccountResponse 
{
    id: u32,
}

#[derive(Deserialize)]
struct EpisodeResponse 
{
    uuid: String,
    title: String,
    episode_number: u32,
    season_number: u32, // Full string like "EN - Stranger Things - S01E01 - The Vanishing of Will Byers"
    duration_secs: Option<u64>, // Note: Can be null/None
    container_extension: String // "mp4" or "mkv" — varies per episode, must be respected
}

/////////////////////////////////////////////////////
// RetrieveError
/////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub enum RetrieveError
{
    FailedToSetupHTTP{ error_type: String },
    GETProviderInfoFailed{ error_type: String },
    ProviderInfoReturnedErrorStatus { status_code: reqwest::StatusCode, error_type: String },
    ProviderInfoContainsNoBody,
    FailedToParseJSON{ error_type: String },
}

impl fmt::Display for RetrieveError
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result 
    {
        match self
        {
            RetrieveError::FailedToSetupHTTP{ error_type } => { write!(formatter, "Failed to set up HTTP client with error: {}.", error_type) },
            RetrieveError::GETProviderInfoFailed{ error_type } => { write!(formatter, "Failed to retrieve episodes from Dispatcharr with error: {}.", error_type) },
            RetrieveError::ProviderInfoReturnedErrorStatus{ status_code, error_type } => { write!(formatter, "Failed to retrieve episodes from Dispatcharr with error code: {}. Additional context: {}.", status_code.as_u16(), error_type) },
            RetrieveError::ProviderInfoContainsNoBody => { write!(formatter, "Failed to retrieve episodes from Dispatcharr, because response does not contain HTTP body.") },
            RetrieveError::FailedToParseJSON{ error_type } => { write!(formatter, "Failed to parse episode response JSON, error: {}.", error_type) },
        }
    }   
}

/////////////////////////////////////////////////////
// Retrieval
/////////////////////////////////////////////////////
pub fn retrieve_episodes(options: &DownloadOptions) -> Result<(Seasons, M3UID), RetrieveError>
{
    // HTTP side
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|error| { return RetrieveError::FailedToSetupHTTP{ error_type: error.to_string() }; })?;

    let url = format!("{}/api/vod/series/{}/provider-info/?include_episodes=true", options.url, options.recipe.series_id);

    trace!("GET URL: {}", url);

    let response: reqwest::blocking::Response = client
        .get(&url)
        .header("X-Api-Key", options.api_key.as_str())
        .send()
        .map_err(|error| { return RetrieveError::GETProviderInfoFailed{ error_type: error.to_string() }; })?;

    trace!("Response: {:?}", response);

    let status = response.status();

    let info = response.error_for_status()
        .map_err(|error| { return RetrieveError::ProviderInfoReturnedErrorStatus{ status_code: status, error_type: error.to_string() }; })?;
    let body = info.text()
        .map_err(|_error| { return RetrieveError::ProviderInfoContainsNoBody; })?;

    trace!("Response body: {:?}", body);

    let json = serde_json::from_str::<ProviderInfoResponse>(body.as_str())
        .map_err(|error| { return RetrieveError::FailedToParseJSON{ error_type: error.to_string() }; })?;

    // Conversion side
    let m3u_account_id = json.m3u_account.id;
    let mut seasons: Seasons = Seasons::new();
    
    for (_season_key, season_episodes) in json.episodes 
    {
        for episode in season_episodes 
        {
            seasons.entry(episode.season_number)
                .or_insert(Season { 
                    episodes: Vec::new()
                })
                .episodes.push(Episode { 
                    uuid: episode.uuid, 
                    episode_number: episode.episode_number, 
                    title: episode.title,
                    container_extension: episode.container_extension,
                    seconds: episode.duration_secs
                });
        }
    }

    Ok((seasons, m3u_account_id))
}
