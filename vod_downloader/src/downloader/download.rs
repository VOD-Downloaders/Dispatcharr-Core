use std::fmt;
// use std::io::Read;
use std::process::Stdio;
use std::process::Command;

use super::types::*;
use super::super::cli::DownloadOptions;

/////////////////////////////////////////////////////
// DownloadError
/////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub enum DownloadError
{
    NoFFmpeg,
    StartDownloadFailed{ title: String, error_type: String },
    DownloadFailed{ title: String, exit_code: i32 },
}

impl fmt::Display for DownloadError
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result 
    {
        match self
        {
            DownloadError::NoFFmpeg => { write!(formatter, "Failed to find `ffmpeg` in $PATH, please install it.") },
            DownloadError::StartDownloadFailed{ title, error_type } => { write!(formatter, "Starting download: \"{}\" failed with error: {}.", title, error_type) },
            DownloadError::DownloadFailed{ title, exit_code } => { write!(formatter, "Download: \"{}\" exited with exit code: {} and subsequently failed.", title, exit_code) },
        }
    }   
}

/////////////////////////////////////////////////////
// Downloader
/////////////////////////////////////////////////////
pub fn download_episode(options: &DownloadOptions, episode: &Episode, m3u_id: M3UID) -> Result<(), DownloadError>
{
    let which_ffmpeg_status = Command::new("which")
        .arg("ffmpeg")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    
    if which_ffmpeg_status.is_err() || !which_ffmpeg_status.unwrap().success() {
        return Err(DownloadError::NoFFmpeg);
    }

    info!("Starting download for episode: \"{}\".", episode.title);

    let url = format!("{}/proxy/vod/episode/{}?m3u_account_id={}", options.url, episode.uuid, m3u_id);
    let output_file = format!("{}.{}", episode.title.chars().filter(|c| !c.is_whitespace()).collect::<String>(), episode.container_extension);

    let mut last_error: DownloadError = DownloadError::NoFFmpeg; // Must be initialized
    for attempt in 1..=options.max_reties 
    {
        match run_ffmpeg_attempt(&url, &output_file, episode.title.as_str()) 
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

fn run_ffmpeg_attempt(url: &str, output_file: &str, debug_title: &str) -> Result<(), DownloadError>
{
    let mut child = Command::new("ffmpeg")
        .arg("-y")
        .arg("-i")
        .arg(url)
        .arg("-c")
        .arg("copy")
        .arg("-bsf:a")
        .arg("aac_adtstoasc")
        .arg(output_file)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| { return DownloadError::StartDownloadFailed{ title: debug_title.to_string(), error_type: e.kind().to_string() }; })?;

    let status = child
        .wait()
        .map_err(|e| { return DownloadError::StartDownloadFailed{ title: debug_title.to_string(), error_type: e.kind().to_string() }; })?;

    if !status.success() {
        return Err(DownloadError::DownloadFailed{ title: debug_title.to_string(), exit_code: status.code().unwrap_or(-1) });
    }

    Ok(())
}