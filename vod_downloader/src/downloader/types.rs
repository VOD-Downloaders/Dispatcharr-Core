use std::collections::HashMap;

/////////////////////////////////////////////////////
// Episodes
/////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub struct Episode
{
    pub uuid: String,
    // pub episode_number: u32,
    pub title: String,
    pub container_extension: String,
    pub seconds: u64
}

#[derive(Debug, Clone)]
pub struct Season
{
    pub episodes: Vec<Episode>
}

// Note: Indexed by 
pub type Seasons = HashMap<u32, Season>;
pub type M3UID = u32;