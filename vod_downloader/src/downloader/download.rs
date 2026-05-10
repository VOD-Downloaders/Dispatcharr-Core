use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use reqwest::blocking::Client;
use mp4::Mp4Reader;

use super::types::*;
use super::super::cli::OverwriteMode;
use super::super::cli::DownloadOptions;

/////////////////////////////////////////////////////
// DownloadError
/////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub enum DownloadError
{
    StartDownloadFailed{ title: String, error_type: String },
    DownloadFailed{ title: String, exit_code: i32 },
    FailedToCreateFile{ title: String, file: PathBuf, error_type: String },
    FailedToCopyContentsToFile{ title: String, file: PathBuf, error_type: String },
    FailedToReadFile{ title: String, error_type: String },
    FailedToReadFileMetadata{ title: String, error_type: String },
    FailedToReadMP4{ title: String, error_type: String },
    FailedToReadMKV{ title: String, error_type: String },
    FailedToRetrieveDuration{ title: String },
    ValidationFailed{ title: String, expected_secs: u64, actual_secs: u64 }
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
            DownloadError::FailedToReadFile{ title,error_type } => { write!(formatter, "Download: \"{}\" failed, due to not being able to validate, because or read error: {}.", title, error_type) },
            DownloadError::FailedToReadFileMetadata{ title, error_type } => { write!(formatter, "Download: \"{}\" failed, due to not being able to validate, because or read metadata error: {}.", title, error_type) },
            DownloadError::FailedToReadMP4{ title, error_type } => { write!(formatter, "Download: \"{}\" failed, due to not being able to read the MP4 (corrupt?), error: {}.", title, error_type) },
            DownloadError::FailedToReadMKV{ title, error_type } => { write!(formatter, "Download: \"{}\" failed, due to not being able to read the MKV (corrupt?), error: {}.", title, error_type) },
            DownloadError::FailedToRetrieveDuration{ title } => { write!(formatter, "Download: \"{}\" failed, due to not being able to read duration.", title) },
            DownloadError::ValidationFailed{ title, expected_secs, actual_secs } => { write!(formatter, "Download: \"{}\" failed, expected file to be {} seconds long, got {} seconds.", title, expected_secs, actual_secs) },
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
                    let validation_result = validate_download(output_file.as_path(), episode.container_extension.as_str(), seconds, episode.title.as_str());
                    match validation_result
                    {
                        Ok(_) => {
                            info!("Episode \"{}\" already exists on disk, OverwriteMode::Bad selected, the episode has been fully validated, so skipping this episode.", episode.title);
                            return Ok(());
                        }
                        Err(error) => {
                            if let DownloadError::ValidationFailed { title: _, expected_secs: _, actual_secs: _ } = error {
                                warning!("Episode \"{}\" already exists on disk, OverwriteMode::Bad selected, this episode failed validation, so overwriting.", episode.title);
                            } else {
                                error!("Episode \"{}\" already exists on disk, OverwriteMode::Bad selected, this episode failed validation with error: \"{}\", so overwriting.", episode.title, error);
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
        validate_download(output_file, container_extension, seconds, debug_title)
    }
    else 
    {
        warning!("Unable to validate episode \"{}\", no duration retrieved from HTTP GET.", debug_title);
        Ok(())
    }
}

fn validate_download(output_file: &Path, container_extension: &str, expected_secs: u64, debug_title: &str) -> Result<(), DownloadError>
{
    const TOLERANCE: u64 = 2; // 2 seconds
    
    match container_extension
    {
        "mp4" | "m4v" | "mov" => validate_mp4(output_file, expected_secs, debug_title, TOLERANCE),
        "mkv" | "webm" => validate_mkv(output_file, expected_secs, debug_title, TOLERANCE),
        _ => {
            warning!("Unable to validate \"{}\", unsupported container type: {}.", debug_title, container_extension);
            Ok(())
        }
    }
}

fn validate_mp4(output_file: &Path, expected_secs: u64, debug_title: &str, tolerance_secs: u64) -> Result<(), DownloadError>
{
    let file = File::open(output_file)
        .map_err(|error| { return DownloadError::FailedToReadFile { title: debug_title.to_string(), error_type: error.to_string() } })?;
    let size = file.metadata()
        .map_err(|error| { return DownloadError::FailedToReadFileMetadata { title: debug_title.to_string(), error_type: error.to_string() } })?
        .len();
    let reader = BufReader::new(file);

    let mp4 = Mp4Reader::read_header(reader, size)
        .map_err(|error| { return DownloadError::FailedToReadMP4 { title: debug_title.to_string(), error_type: error.to_string() } })?;

    let actual_secs = mp4.duration().as_secs();
    let delta = actual_secs.abs_diff(expected_secs);

    if delta > tolerance_secs {
        return Err(DownloadError::ValidationFailed { title: debug_title.to_string(), expected_secs: expected_secs, actual_secs: actual_secs } );
    }

    Ok(())
}

fn validate_mkv(output_file: &Path, expected_secs: u64, debug_title: &str, tolerance_secs: u64) -> Result<(), DownloadError>
{
    let file = File::open(output_file)
        .map_err(|error| { return DownloadError::FailedToReadFile { title: debug_title.to_string(), error_type: error.to_string() } })?;

    let mkv = matroska::Matroska::open(file)
        .map_err(|error| { return DownloadError::FailedToReadMKV { title: debug_title.to_string(), error_type: error.to_string() }})?;

    let Some(actual_secs) = mkv.info.duration else {
        return Err(DownloadError::FailedToRetrieveDuration { title: debug_title.to_string() });
    };
    let delta = actual_secs.as_secs().abs_diff(expected_secs);

    if delta > tolerance_secs {
        return Err(DownloadError::ValidationFailed { title: debug_title.to_string(), expected_secs: expected_secs, actual_secs: actual_secs.as_secs()} );
    }

    Ok(())
}