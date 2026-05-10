use std::fmt;
use std::path::PathBuf;

use super::arguments::CliOption;

use super::super::recipe;
use super::super::recipe::Recipe;

/////////////////////////////////////////////////////
// CliError
/////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub enum CliError
{
    UnknownFlag{ flag: String },
    UnknownOption{ option: String },

    NoUrl,
    InvalidUrl{ message: String },
    NoSeriesId,
    NonExistentRecipeFile{ file: PathBuf },
    RecipeParseError{ parse_error: recipe::ParseError },
    NoApiKey,
    InvalidOutputPath{ message: String },
    InvalidLogFile{ message: String },
    InvalidRetryCount{ message: String },
}

impl fmt::Display for CliError
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result 
    {
        match self
        {
            CliError::UnknownFlag{ flag } => { write!(formatter, "Unknown flag: '-{}'.", flag) },
            CliError::UnknownOption{ option } => { write!(formatter, "Unknown option: '--{}=...'.", option) },
            CliError::NoUrl => { write!(formatter, "No url passed in. (use --url=...).") },
            CliError::InvalidUrl{ message} => { write!(formatter, "Url passed in is invalid, {}.", message) },
            CliError::NoSeriesId => { write!(formatter, "No series id in your recipe.") },
            CliError::NonExistentRecipeFile{ file } => { write!(formatter, "Recipe file passed in (\"{}\") does not exist.", file.display()) },
            CliError::RecipeParseError{ parse_error } => { write!(formatter, "Failed to create recipe with error: {}", parse_error) },
            CliError::NoApiKey => { write!(formatter, "No api key passed in. (use --api-key=...).") },
            CliError::InvalidOutputPath{ message } => { write!(formatter, "Invalid output path passed in, {}.", message) },
            CliError::InvalidLogFile{ message } => { write!(formatter, "Invalid log file path passed in, {}.", message) },
            CliError::InvalidRetryCount{ message } => { write!(formatter, "Invalid retry count passed in, {}.", message) },
        }
    }   
}

/////////////////////////////////////////////////////
// DownloadOptions
/////////////////////////////////////////////////////
#[derive(Debug, Clone, PartialEq)]
pub enum OverwriteMode
{
    None,
    Bad,
    All,
}

#[derive(Debug, Clone)]
pub struct DownloadOptions
{
    pub url: String, // ex. http://192.168.2.2:9191 or https://dispatcharr.example.com
    pub api_key: String,
    
    pub output_folder: PathBuf,
    
    pub log_file: Option<PathBuf>,

    pub recipe: Recipe,
    
    pub max_reties: u32,
    pub overwrite_mode: OverwriteMode,
    pub verbose: bool,
}

/////////////////////////////////////////////////////
// Parse functions
/////////////////////////////////////////////////////
pub fn parse_cli_options(cli_options: Vec<CliOption>) -> Result<DownloadOptions, CliError>
{
    let mut options: DownloadOptions = DownloadOptions { 
        url: String::new(),
        api_key: String::new(),

        output_folder: PathBuf::from("."),

        log_file: None,

        recipe: Recipe::new(),

        max_reties: 3,
        overwrite_mode: OverwriteMode::Bad,
        verbose: false
    };

    let mut i: usize = 0;
    while i < cli_options.len()
    {
        match cli_options.get(i).unwrap()
        {
            CliOption::Value(value) => 
            {
                options.recipe = parse_recipe(value)?;
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

                                "log" => { options.log_file = Some(parse_output_folder(flag_value)?); },
                                "logfile" => { options.log_file = Some(parse_output_folder(flag_value)?); },
                                _ => { panic!("Internal logic error, a flag was set in top level match statement but not in bottom level."); }
                            }

                            // Skip the next value
                            i += 1;
                        }
                    }

                    "v" => { options.verbose = true; },
                    "verbose" => { options.verbose = true; },
                    "d" => { options.verbose = true; },

                    "debug" => { options.verbose = true; },

                    "overwrite-none" => { options.overwrite_mode = OverwriteMode::None; },
                    "overwrite-bad" => { options.overwrite_mode = OverwriteMode::Bad; },
                    "overwrite-all" => { options.overwrite_mode = OverwriteMode::All; },

                    _ => { return Err(CliError::UnknownFlag{ flag: value.to_string() }); }
                }
            },
            CliOption::Option(key, value) =>
            {
                match key.as_str()
                {
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

                    "log" => { options.log_file = Some(parse_log_file(value)?); },
                    "logfile" => { options.log_file = Some(parse_log_file(value)?); },
                    "log-file" => { options.log_file = Some(parse_log_file(value)?); },
                    "log_file" => { options.log_file = Some(parse_log_file(value)?); },

                    "retries" => { options.max_reties = parse_max_retries(value)?; },
                    "maxretries" => { options.max_reties = parse_max_retries(value)?; },
                    "max-retries" => { options.max_reties = parse_max_retries(value)?; },
                    "max_retries" => { options.max_reties = parse_max_retries(value)?; },
                    
                    _ => { return Err(CliError::UnknownOption{ option: key.to_string() }); }
                }
            }
        }

        i += 1;
    }

    if options.recipe.series_id == 0 { return Err(CliError::NoSeriesId); }
    if options.url == "" { return Err(CliError::NoUrl); }
    if options.api_key == "" { return Err(CliError::NoApiKey); }
    if options.max_reties == 0 { return Err(CliError::InvalidRetryCount{ message: "retry count must be greater than zero".to_string() }); }

    Ok(options)
}

fn parse_recipe(file: &str) -> Result<Recipe, CliError>
{
    let path = file.parse::<PathBuf>().unwrap();

    if !path.is_file()
    {
        Err(CliError::NonExistentRecipeFile{ file: path })
    }
    else 
    {
        let recipe = recipe::parse_recipe(path.as_path())
            .map_err(|error| { return CliError::RecipeParseError{ parse_error: error} })?;
        
        Ok(recipe)
    }
}

fn parse_url(url: &str) -> Result<String, CliError>
{
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(CliError::InvalidUrl{ message: "the URL passed in doens't start with http or https".to_string() });
    }
    if url.split('.').count() < 2 {
        return Err(CliError::InvalidUrl{ message: "not enough .'s to form a valid URL".to_string() });
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
        Err(CliError::InvalidOutputPath{ message: "expected a folder as an output destination".to_string() })
    }
    else 
    {
        Ok(path)    
    }
}

fn parse_log_file(log_file: &str) -> Result<PathBuf, CliError>
{
    let path = log_file.parse::<PathBuf>().unwrap();

    if log_file.ends_with(std::path::MAIN_SEPARATOR)
    {
        Err(CliError::InvalidLogFile{ message: "expected a file as a log destination".to_string() })
    }
    else 
    {
        Ok(path)    
    }
}

fn parse_max_retries(max_retries: &str) -> Result<u32, CliError>
{
    Ok(max_retries.parse::<u32>().map_err(|_error| { return CliError::InvalidRetryCount{ message: format!("'{}' is not an integer.", max_retries) } })?)
}