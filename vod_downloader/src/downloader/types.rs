use std::collections::HashMap;

/////////////////////////////////////////////////////
// Episodes
/////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub struct Episode
{
    pub uuid: String,
    pub episode_number: u32,
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct Season
{
    pub season: u32,
    pub episodes: Vec<Episode>
}

// Note: Indexed by 
pub type Episodes = HashMap<u32, Season>;
pub type M3UID = u32;