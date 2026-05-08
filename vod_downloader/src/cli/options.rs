use std::path::PathBuf;

use super::arguments::CliOption;

/////////////////////////////////////////////////////
// CliError
/////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub enum CliError
{
    UnknownFlag(String),
    UnknownOption(String),

    NoUrl,
    InvalidUrl(String),
    NoSeriesId,
    InvalidSeriesId(String),
    NoApiKey,
    InvalidPath(String),
    InvalidRetryCount(String),
}

/////////////////////////////////////////////////////
// CompilerOptions
/////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub struct DownloadOptions
{
    pub url: String, // ex. http://192.168.2.2:9191 or https://dispatcharr.example.com
    pub series_id: u32,
    pub api_key: String,
    pub output_folder: PathBuf,
    pub log_file: PathBuf,
    pub max_reties: u32,
    // TODO: Add more specifiers about what seasons or specific episodes
    
    pub verbose: bool,
}

/////////////////////////////////////////////////////
// Parse functions
/////////////////////////////////////////////////////
pub fn parse_cli_options(cli_options: Vec<CliOption>) -> Result<DownloadOptions, CliError>
{
    let mut options: DownloadOptions = DownloadOptions { 
        url: String::new(),
        series_id: 0,
        api_key: String::new(),
        output_folder: PathBuf::from("."),
        log_file: PathBuf::from("vod_download.log"),
        max_reties: 3,
        verbose: false
    };

    let mut i: usize = 0;
    while i < cli_options.len()
    {
        match cli_options.get(i).unwrap()
        {
            CliOption::Value(value) => 
            {
                options.series_id = parse_series_id(value)?;
            },
            CliOption::Flag(value) => 
            {
                match value.as_str()
                {
                    "u" | "url" | "baseurl" |
                    "o" | "output" | "outputfolder" | 
                    "log" | "logfile" => 
                    {
                        if let Some(next) = cli_options.get(i + 1) && let CliOption::Value(flag_value) = next
                        {
                            match value.as_str()
                            {
                                "u" => { options.url = parse_url(flag_value)?; },
                                "baseurl" => { options.url = parse_url(flag_value)?; },

                                "o" => { options.output_folder = parse_output_folder(flag_value)?; },
                                "output" => { options.output_folder = parse_output_folder(flag_value)?; },
                                "outputfolder" => { options.output_folder = parse_output_folder(flag_value)?; },

                                "log" => { options.log_file = parse_output_folder(flag_value)?; },
                                "logfile" => { options.log_file = parse_output_folder(flag_value)?; },
                                _ => { panic!("Internal logic error, a flag was set in top level match statement but not in bottom level."); }
                            }

                            // Skip the next value
                            i += 1;
                        }
                    }

                    "v" => { options.verbose = true; },
                    "verbose" => { options.verbose = true; },

                    _ => { return Err(CliError::UnknownFlag(format!("Unknown flag: '-{}'", value))); }
                }
            },
            CliOption::Option(key, value) =>
            {
                match key.as_str()
                {
                    "series" => { options.series_id = parse_series_id(value)?; },
                    "seriesid" => { options.series_id = parse_series_id(value)?; },
                    "series-id" => { options.series_id = parse_series_id(value)?; },
                    "series_id" => { options.series_id = parse_series_id(value)?; },
                    
                    "url" => { options.url = parse_url(value)?; },
                    "baseurl" => { options.url = parse_url(value)?; },
                    "base-url" => { options.url = parse_url(value)?; },
                    "base_url" => { options.url = parse_url(value)?; },
                    
                    "api" => { options.api_key = parse_api_key(value)?; },
                    "apikey" => { options.api_key = parse_api_key(value)?; },
                    "api-key" => { options.api_key = parse_api_key(value)?; },
                    "api_key" => { options.api_key = parse_api_key(value)?; },
                    
                    "output" => { options.output_folder = parse_output_folder(value)?; },
                    "outputfolder" => { options.output_folder = parse_output_folder(value)?; },
                    "output-folder" => { options.output_folder = parse_output_folder(value)?; },
                    "output_folder" => { options.output_folder = parse_output_folder(value)?; },

                    "log" => { options.log_file = parse_log_file(value)?; },
                    "logfile" => { options.log_file = parse_log_file(value)?; },
                    "log-file" => { options.log_file = parse_log_file(value)?; },
                    "log_file" => { options.log_file = parse_log_file(value)?; },

                    "retries" => { options.max_reties = parse_max_retries(value)?; },
                    "maxretries" => { options.max_reties = parse_max_retries(value)?; },
                    "max-retries" => { options.max_reties = parse_max_retries(value)?; },
                    "max_retries" => { options.max_reties = parse_max_retries(value)?; },
                    
                    _ => { return Err(CliError::UnknownOption(format!("Unknown option: '--{}=...'", key))); }
                }
            }
        }

        i += 1;
    }

    if options.series_id == 0 { return Err(CliError::NoSeriesId); }
    if options.url == "" { return Err(CliError::NoUrl); }
    if options.api_key == "" { return Err(CliError::NoApiKey); }
    if options.max_reties == 0 { return Err(CliError::InvalidRetryCount("Retry count must be greater than zero.".to_string())); }

    Ok(options)
}

fn parse_series_id(series_id: &str) -> Result<u32, CliError>
{
    Ok(series_id.parse::<u32>().map_err(|_error| { return CliError::InvalidSeriesId(format!("'{}' is not an integer.", series_id))})?)
}

fn parse_url(url: &str) -> Result<String, CliError>
{
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(CliError::InvalidUrl(format!("The URL passed in doens't start with http or https, passed in: '{}'.", url)));
    }
    if url.split('.').count() < 2 {
        return Err(CliError::InvalidUrl(format!("Invalid url format: '{}'.", url)));
    }

    if url.ends_with('/')
    {
        Ok(url[0..url.len()-2].to_string())
    }
    else 
    {
        Ok(url.to_string())
    }
}

fn parse_api_key(api_key: &str) -> Result<String, CliError>
{
    Ok(api_key.to_string())
}

fn parse_output_folder(output_folder: &str) -> Result<PathBuf, CliError>
{
    let path = output_folder.parse::<PathBuf>().unwrap();

    if !path.is_dir() 
    {
        Err(CliError::InvalidPath("Expected a folder as an output destination.".to_string()))
    }
    else 
    {
        Ok(path)    
    }
}

fn parse_log_file(log_file: &str) -> Result<PathBuf, CliError>
{
    let path = log_file.parse::<PathBuf>().unwrap();

    if !path.is_file() 
    {
        Err(CliError::InvalidPath("Expected a file as a log destination.".to_string()))
    }
    else 
    {
        Ok(path)    
    }
}

fn parse_max_retries(max_retries: &str) -> Result<u32, CliError>
{
    Ok(max_retries.parse::<u32>().map_err(|_error| { return CliError::InvalidRetryCount(format!("'{}' is not an integer.", max_retries))})?)
}