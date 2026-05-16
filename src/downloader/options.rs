use std::path::PathBuf;

use super::super::recipe::Recipe;

/////////////////////////////////////////////////////
// DownloadOptions
/////////////////////////////////////////////////////
#[derive(Debug, Clone, PartialEq)]
pub enum OverwriteMode {
    None,
    Bad,
    All,
}

#[derive(Debug, Clone)]
pub struct DownloadOptions {
    pub url: String, // ex. http://192.168.2.2:9191 or https://dispatcharr.example.com
    pub api_key: String,

    pub output_folder: PathBuf,

    pub log_file: Option<PathBuf>,

    pub recipe: Recipe,

    pub max_reties: u32,
    pub overwrite_mode: OverwriteMode,
    pub verbose: bool,
}
