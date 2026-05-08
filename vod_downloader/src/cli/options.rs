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
    InvalidSeriesId(String)
}

/////////////////////////////////////////////////////
// CompilerOptions
/////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub struct DownloadOptions
{
    pub url: String, // ex. http://192.168.2.2:9191 or https://dispatcharr.example.com
    pub series_id: u32
    // TODO: Add more specifiers about what seasons or specific episodes
}

/////////////////////////////////////////////////////
// Parse functions
/////////////////////////////////////////////////////
pub fn parse_cli_options(cli_options: Vec<CliOption>) -> Result<DownloadOptions, CliError>
{
    let mut options: DownloadOptions = DownloadOptions { 
        url: String::new(),
        series_id: 0
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
                    "u" => { options.url = parse_url(value)?; }
                    _ => { return Err(CliError::UnknownFlag(format!("Unknown flag: '-{}'", value))); }
                }
            },
            CliOption::Option(key, value) =>
            {
                match key.as_str()
                {
                    "series" => { options.series_id = parse_series_id(value)?; },
                    "url" => { options.url = parse_url(value)?; },
                    _ => { return Err(CliError::UnknownOption(format!("Unknown option: '--{}=...'", key))); }
                }
            }
        }

        i += 1;
    }

    if options.series_id == 0 { return Err(CliError::NoSeriesId); }
    if options.url == "" { return Err(CliError::NoUrl); }

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