use std::io::BufRead;
use std::io::BufReader;
use std::process::Command;

use super::cli::DownloadOptions;

mod types;
mod retrieval;

pub use types::*;
pub use retrieval::*;

/////////////////////////////////////////////////////
// DownloadError
/////////////////////////////////////////////////////
pub enum DownloadError
{
    NoFFmpeg,
    CommandFailed(String),
}

/////////////////////////////////////////////////////
// Downloader
/////////////////////////////////////////////////////
pub fn download_episode(options: &DownloadOptions, episode: &Episode, m3u_id: M3UID, follow_output: bool) -> Result<String, DownloadError>
{
    if which::which("ffmpeg").is_err() {
        return Err(DownloadError::NoFFmpeg);
    }

    let url = format!("{}/proxy/vod/episode/{}?m3u_account_id={}", options.url, episode.uuid, m3u_id);

    let mut command = Command::new("ffmpeg")
        .arg("-i")
        .arg(format!("{}", url).as_str())
        .arg("-c")
        .arg("copy")
        .arg("-bsf:a")
        .arg("aac_adtstoasc")
        .arg(format!("{}.mp4", episode.title.chars().filter(|c| !c.is_whitespace()).collect::<String>()).as_str())
        .spawn()
        .map_err(|error| { return DownloadError::CommandFailed(format!("{}", error.kind())) })?;
        // TODO: Respect container mp4/mkv type

    if follow_output
    {
        // TODO: into string for output at the end
        if let Some(stdout) = command.stdout.take() 
        {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                println!("Live Output: {}", line.unwrap());
            }
        }
    }

    command.wait().map_err(|error| { return DownloadError::CommandFailed(format!("Command failed with exit code: {}", error.kind())) })?;
        
    match command.stdout
    {
        Some(stdout) => {
            let mut output = String::new(); 
            let reader = BufReader::new(stdout);

            for line in reader.lines() {
                output.push_str(line.unwrap_or("<FAILED TO READ LINE>".to_string()).as_str());
            }

            Ok(output)
        },
        None => { Ok("<NO OUTPUT>".to_string()) }
    }
}