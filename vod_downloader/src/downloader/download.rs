use std::fmt;
use std::io::Read;
use std::io::Write;
use std::process::Stdio;
use std::process::Command;
use std::fs::File;

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
pub fn download_episode(options: &DownloadOptions, episode: &Episode, m3u_id: M3UID, log_file: &mut File) -> Result<String, DownloadError>
{
    let which_ffmpeg_status = Command::new("which")
        .arg("ffmpeg")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    
    if which_ffmpeg_status.is_err() || !which_ffmpeg_status.unwrap().success() {
        return Err(DownloadError::NoFFmpeg);
    }

    let url = format!("{}/proxy/vod/episode/{}?m3u_account_id={}", options.url, episode.uuid, m3u_id);
    let output_file = format!("{}.{}", episode.title.chars().filter(|c| !c.is_whitespace()).collect::<String>(), episode.container_extension);

    let mut last_error: DownloadError = DownloadError::NoFFmpeg; // Must be initialized
    for attempt in 1..=options.max_reties 
    {
        // Write the attempt header
        let separator = "=".repeat(80);
        let header = format!(
            "\n{sep}\n* {title} — Attempt {attempt}/{max}\n{sep}\n",
            sep = separator,
            title = episode.title,
            attempt = attempt,
            max = options.max_reties,
        );
        let _ = log_file.write_all(header.as_bytes());

        match run_ffmpeg_attempt(&url, &output_file, log_file, episode.title.as_str()) 
        {
            Ok(output) => return Ok(output),
            Err(e) => 
            {
                eprintln!("[Attempt {}/{}] Failed: {:?}", attempt, options.max_reties, e);
                last_error = e;
            }
        }
    }

    Err(last_error)
}

fn run_ffmpeg_attempt(url: &str, output_file: &str, log_file: &mut File, debug_title: &str) -> Result<String, DownloadError>
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

    let mut output: String = String::new();
    child.stdout.unwrap().read_to_string(&mut output); // TODO: Remove unsafe .unwrap()

    log_file.write_all(output.as_bytes()); // TODO: result

    Ok(output)
}