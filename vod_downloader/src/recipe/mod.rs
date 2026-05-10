use std::fmt;
use std::fs::File;
// use std::fs::OpenOptions;
use std::io::Read;
// use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::collections::HashMap;
use serde::Serialize;
use serde::Deserialize;

/////////////////////////////////////////////////////
// Recipe
/////////////////////////////////////////////////////
#[derive(Debug, Serialize, Deserialize)]
pub struct Season
{
    pub episodes: Vec<u32>,
    pub exclude: Vec<u32>
} 

#[derive(Debug, Serialize, Deserialize)]
pub struct Recipe 
{
    pub series_id: u64,
    pub seasons: HashMap<u32, Season>
}

/////////////////////////////////////////////////////
// ParseError
/////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub enum ParseError
{
    FailedToOpenFile{ file: PathBuf, error_type: String },
    FailedToReadFile{ file: PathBuf, error_type: String },
    FailedToParseJSON{ error_type: String },
}

impl fmt::Display for ParseError
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result 
    {
        match self
        {
            ParseError::FailedToOpenFile{ file, error_type} => { write!(formatter, "Failed to open \"{}\", error: {}.", file.display(), error_type) },
            ParseError::FailedToReadFile{ file, error_type} => { write!(formatter, "Failed to read \"{}\", error: {}.", file.display(), error_type) },
            ParseError::FailedToParseJSON{ error_type} => { write!(formatter, "Failed to parse Recipe JSON, error: {}.", error_type) },
        }
    }   
}

/////////////////////////////////////////////////////
// Parse
/////////////////////////////////////////////////////
pub fn parse_recipe(path: &Path) -> Result<Recipe, ParseError>
{
    let mut file = File::open(path)
        .map_err(|error| { return ParseError::FailedToOpenFile { file: path.to_path_buf(), error_type: error.to_string() } })?;

    let mut json: String = String::new();
    file.read_to_string(&mut json)
        .map_err(|error| { return ParseError::FailedToReadFile { file: path.to_path_buf(), error_type: error.to_string() }; })?;

    let recipe = serde_json::from_str::<Recipe>(json.as_str())
        .map_err(|error| { return ParseError::FailedToParseJSON { error_type: error.to_string() }; })?;

    Ok(recipe)
}

// fn dump_recipe(recipe: &Recipe, path: &Path)
// {
//     let mut file = OpenOptions::new()
//         .create(true)
//         .write(true)
//         .open(path)
//         .unwrap();
// 
//     let value = serde_json::to_string::<Recipe>(recipe).unwrap();
// 
//     file.write_fmt(format_args!("{}", value));
// }