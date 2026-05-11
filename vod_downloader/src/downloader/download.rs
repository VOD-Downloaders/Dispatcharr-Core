use std::fmt;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

use reqwest::blocking::Client;

use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatOptions, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::Time;

use super::types::*;
use super::super::cli::OverwriteMode;
use super::super::cli::DownloadOptions;

/////////////////////////////////////////////////////
// DownloadError & ValidationError
/////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub enum DownloadError
{
    StartDownloadFailed{ title: String, error_type: String },
    DownloadFailed{ title: String, exit_code: i32 },
    FailedToCreateFile{ title: String, file: PathBuf, error_type: String },
    FailedToCopyContentsToFile{ title: String, file: PathBuf, error_type: String },
    ValidationFailed{ title: String, error: ValidationError },
}

impl fmt::Display for DownloadError
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result 
    {
        match self
        {
            DownloadError::StartDownloadFailed{ title, error_type } => { write!(formatter, "Starting download: \"{}\" failed with error: {}.", title, error_type) },
            DownloadError::DownloadFailed{ title, exit_code } => { write!(formatter, "Download: \"{}\" exited with exit code: {} and subsequently failed.", title, exit_code) },
            DownloadError::FailedToCreateFile{ title, file, error_type } => { write!(formatter, "Download: \"{}\" failed, because of being unable to create file \"{}\" due to error: {}.", title, file.display(), error_type) },
            DownloadError::FailedToCopyContentsToFile{ title, file, error_type } => { write!(formatter, "Download: \"{}\" failed, because of being unable to copy HTTP response contents to file \"{}\" with errorcode: {}.", title, file.display(), error_type) },
            DownloadError::ValidationFailed{ title, error } => { write!(formatter, "Download: \"{}\" failed during validation, error: {}.", title, error) },
        }
    }   
}

#[derive(Debug, Clone)]
pub enum ValidationError
{
    UnsupportedContainerType{ container_type: String },
    FailedToReadFile{ error_type: String },
    FailedToGetFormat{ error_type: String },
    NoTrackFound,
    PacketReadError{ error_type: String },
    DurationMismatch{ expected_secs: u64, actual_secs: u64 }
}

impl fmt::Display for ValidationError
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result 
    {
        match self
        {
            ValidationError::UnsupportedContainerType{ container_type } => { write!(formatter, "Container type not supported for validating: {}.", container_type) },
            ValidationError::FailedToReadFile{ error_type } => { write!(formatter, "Read error: {}.",error_type) },
            ValidationError::FailedToGetFormat{ error_type } => { write!(formatter, "Unable to read video format, error: {}.", error_type) },
            ValidationError::NoTrackFound => { write!(formatter, "Unable to find a video track in the file.") },
            ValidationError::PacketReadError{ error_type } => { write!(formatter, "Unable to read packets, error: {}.", error_type) },
            ValidationError::DurationMismatch{ expected_secs, actual_secs } => { write!(formatter, "Duration mismatch, expected file to be {} seconds long, got {} seconds.", expected_secs, actual_secs) },
        }
    }   
}

/////////////////////////////////////////////////////
// Downloader
/////////////////////////////////////////////////////
pub fn download_episode(options: &DownloadOptions, episode: &Episode, m3u_id: M3UID) -> Result<(), DownloadError>
{
    let url = format!("{}/proxy/vod/episode/{}?m3u_account_id={}", options.url, episode.uuid, m3u_id);
    let file_name = format!("{}.{}", episode.title.chars().filter(|c| !c.is_whitespace()).collect::<String>(), episode.container_extension);
    let output_file: PathBuf = options.output_folder.join(PathBuf::from(file_name));

    // Handle overwrites
    if output_file.exists() 
    {
        match options.overwrite_mode
        {
            OverwriteMode::None => {
                info!("Episode \"{}\" already exists on disk, OverwriteMode::None selected, so skipping this episode.", episode.title);
                return Ok(());
            },
            OverwriteMode::Bad => {
                if let Some(seconds) = episode.seconds
                {
                    let validation_result = validate_download(output_file.as_path(), episode.container_extension.as_str(), seconds);
                    match validation_result
                    {
                        Ok(_) => {
                            info!("Episode \"{}\" already exists on disk, OverwriteMode::Bad selected, the episode has been fully validated, so skipping this episode.", episode.title);
                            return Ok(());
                        }
                        Err(error) => {
                            if let ValidationError::DurationMismatch { expected_secs: _, actual_secs: _ } = error {
                                warning!("Episode \"{}\" already exists on disk, OverwriteMode::Bad selected, this episode failed validation with error: \"{}\", so overwriting.", episode.title, error);
                            } else {
                                warning!("Episode \"{}\" already exists on disk, OverwriteMode::Bad selected, unable to determine if episode on disk is bad, error: {}. So, skipping...", episode.title, error);
                                return Ok(());
                            }
                        }
                    }
                }
            },
            OverwriteMode::All => {
                info!("Episode \"{}\" already exists on disk, OverwriteMode::All selected, so overwriting...", episode.title);
            }
        }
    }

    // Start attempts
    let mut last_error: DownloadError = DownloadError::StartDownloadFailed { title: "".to_string(), error_type: "".to_string() }; // Must be initialized
    for attempt in 1..=options.max_reties 
    {
        match download_attempt(&url, &output_file, episode.container_extension.as_str(), episode.seconds, episode.title.as_str()) 
        {
            Ok(_) => return Ok(()),
            Err(e) => 
            {
                warning!("[Attempt {}/{}] Failed with error: {}.", attempt, options.max_reties, e);
                last_error = e;
            }
        }
    }

    Err(last_error)
}

fn download_attempt(url: &str, output_file: &Path, container_extension: &str, expected_secs: Option<u64>, debug_title: &str) -> Result<(), DownloadError>
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
        .map_err(|e| DownloadError::FailedToCreateFile { title: debug_title.to_string(), file: output_file.to_path_buf(), error_type: e.to_string() })?;

    response.copy_to(&mut file)
        .map_err(|e| DownloadError::FailedToCopyContentsToFile { title: debug_title.to_string(), file: output_file.to_path_buf(), error_type: e.to_string() })?;

    // Validate
    if let Some(seconds) = expected_secs
    {
        let actual_seconds = validate_download(output_file, container_extension, seconds)
            .map_err(|error| { return DownloadError::ValidationFailed { title: debug_title.to_string(), error: error } })?;
        info!("Validation successful for \"{}\": found {}s, expected {}s", debug_title, actual_seconds, seconds);
        Ok(())
    }
    else 
    {
        warning!("Unable to validate episode \"{}\", no duration retrieved from HTTP GET.", debug_title);
        Ok(())
    }
}

fn validate_download(output_file: &Path, container_extension: &str, expected_secs: u64) -> Result<u64, ValidationError>
{
    const VALIDATION_RETRIES: u64 = 3; // 3 validation tries
    const TOLERANCE: u64 = 2; // 2 seconds
    
    match container_extension
    {
        "mp4" | "m4v" | "mov" | "mkv" | "webm" => {
            let last_error = ValidationError::NoTrackFound; // Must be initialized
            
            for i in 1..=VALIDATION_RETRIES
            {
                match validate_mp4_or_mkv(output_file, container_extension, expected_secs, TOLERANCE)
                {
                    Ok(seconds) => { return Ok(seconds); },
                    Err(error) => {
                        match error 
                        {
                            // Definite failure
                            ValidationError::DurationMismatch { expected_secs: _, actual_secs: _ } => { return Err(error); }
                            _ => { // Could be a tiny hiccup
                                warning!("[Validation attempt {}/3] Validation failed with error: {}, retrying.", i, error);
                            }
                        }
                    }
                }
            }

            Err(last_error)
        }
        _ => {
            Err(ValidationError::UnsupportedContainerType { container_type: container_extension.to_string() })
        }
    }
}

fn validate_mp4_or_mkv(path: &Path, container_extension: &str, expected_secs: u64, tolerance_secs: u64) -> Result<u64, ValidationError> 
{
    let file = File::open(path)
        .map_err(|error| ValidationError::FailedToReadFile { error_type: error.to_string() })?;

    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    hint.with_extension(container_extension);

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|error| ValidationError::FailedToGetFormat { error_type: error.to_string() })?;

    let mut format = probed.format;

    // 1. Find the track and check metadata duration first
    let track = format.tracks()
        .iter()
        .find(|t| t.codec_params.time_base.is_some())
        .ok_or(ValidationError::NoTrackFound)?;

    let time_base = track.codec_params.time_base.unwrap();
    let track_id = track.id;

    // Check if the file reports a duration significantly shorter than expected, this prevents "Seek beyond EOF" panic in MKV
    if let Some(n_frames) = track.codec_params.n_frames 
    {
        let metadata_duration = time_base.calc_time(n_frames).seconds;
        
        if metadata_duration < expected_secs.saturating_sub(tolerance_secs) {
            return Err(ValidationError::DurationMismatch { expected_secs, actual_secs: metadata_duration });
        }
    }

    // Start the demuxer
    let _ = format.next_packet(); 

    let seek_target = expected_secs.saturating_sub(30);
    if seek_target > 0 
    {
        // If the seek fails, we don't necessarily error, we just start from the beginning
        let _ = format.seek(SeekMode::Coarse, SeekTo::Time {
                time: Time::new(seek_target, 0.0),
                track_id: None,
            }
        );
    }

    let mut last_secs = 0.0f64;

    loop 
    {
        match format.next_packet() 
        {
            Ok(packet) if packet.track_id() == track_id => {
                let t = time_base.calc_time(packet.ts);
                last_secs = t.seconds as f64 + t.frac;
            }
            Ok(_) => continue,

            Err(SymphoniaError::IoError(_)) | Err(SymphoniaError::ResetRequired) => break,

            Err(error) => return Err(ValidationError::PacketReadError { error_type: error.to_string() }),
        }
    }

    // Final comparison
    let diff = (last_secs - expected_secs as f64).abs();

    if diff <= tolerance_secs as f64 {
        Ok(last_secs as u64)
    } else {
        Err(ValidationError::DurationMismatch { expected_secs, actual_secs: last_secs as u64 })
    }
}