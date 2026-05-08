use std::thread;
use std::sync::Arc;
use std::sync::Mutex;
use std::io::Read;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::process::Stdio;
use std::process::Command;
use std::fs::File;
use std::fs::OpenOptions;
use std::path::PathBuf;

use super::cli::DownloadOptions;

mod types;
mod retrieval;

pub use types::*;
pub use retrieval::*;

/////////////////////////////////////////////////////
// DownloadError
/////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub enum DownloadError
{
    NoFFmpeg,
    CommandFailed(String),
}

/////////////////////////////////////////////////////
// Downloader
/////////////////////////////////////////////////////
pub fn download_episode(options: &DownloadOptions, episode: &Episode, m3u_id: M3UID, log_file: &mut File) -> Result<String, DownloadError>
{
    let which_ffmpeg_status = Command::new("which").arg("ffmpeg").status();
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

        match run_ffmpeg_attempt(&url, &output_file, log_file) 
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

fn run_ffmpeg_attempt(url: &str, output_file: &str, log_file: &mut File) -> Result<String, DownloadError>
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
        .map_err(|e| DownloadError::CommandFailed(format!("Spawn failed with error: {}", e.kind())))?;

    let status = child
        .wait()
        .map_err(|e| DownloadError::CommandFailed(format!("Command failed with error: {}", e.kind())))?;

    if !status.success() {
        return Err(DownloadError::CommandFailed(format!("ffmpeg exited with {}", status.code().unwrap_or(-1))));
    }

    let mut output: String = String::new();
    child.stdout.unwrap().read_to_string(&mut output); // TODO: Remove unsafe .unwrap()

    log_file.write_all(output.as_bytes()); // TODO: result

    Ok(output)
}