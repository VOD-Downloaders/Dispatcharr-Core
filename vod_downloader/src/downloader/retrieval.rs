use serde::Deserialize;
use std::collections::HashMap;

use super::types::*;
use super::super::cli::DownloadOptions;

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
    episode_number: u32,
    season_number: u32, // Full string like "EN - Stranger Things - S01E01 - The Vanishing of Will Byers"
    title: String, // "mp4" or "mkv" — varies per episode, must be respected
    container_extension: String,
}

/////////////////////////////////////////////////////
// RetrieveError
/////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub enum RetrieveError
{
    FailedToSetupHTTP,
    GETProviderInfoFailed,
    ProviderInfoReturnedErrorStatus,
    FailedToParseJSON,
    Other(String)
}

/////////////////////////////////////////////////////
// Retrieval
/////////////////////////////////////////////////////
pub fn retrieve_episodes(options: &DownloadOptions) -> Result<(Episodes, M3UID), RetrieveError>
{
    // HTTP side
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|_error| { return RetrieveError::FailedToSetupHTTP; })?;

    let url = format!("{}/api/vod/series/{}/provider-info/?include_episodes=true", options.url, options.series_id);

    let response: reqwest::blocking::Response = client
        .get(&url)
        .header("X-Api-Key", options.api_key.as_str())
        .send()
        .map_err(|_error| { return RetrieveError::GETProviderInfoFailed; })?;

    let info = response.error_for_status()
        .map_err(|_error| { return RetrieveError::ProviderInfoReturnedErrorStatus; })?;

    let json = info.json::<ProviderInfoResponse>()
        .map_err(|_error| { return RetrieveError::FailedToParseJSON; })?;

    // Conversion side
    let m3u_account_id = json.m3u_account.id;
    let mut episodes: Episodes = Episodes::new();
    
    for (_season_key, season_episodes) in json.episodes 
    {
        for episode in season_episodes 
        {
            episodes.entry(episode.season_number)
                .or_insert(Season { 
                    season: episode.season_number,
                    episodes: Vec::new()
                })
                .episodes.push(Episode { 
                    uuid: episode.uuid, 
                    episode_number: episode.episode_number, 
                    title: episode.title
                });
        }
    }

    //episodes.sort_by_key(|e| (e.season, e.episode));

    Ok((episodes, m3u_account_id))
}