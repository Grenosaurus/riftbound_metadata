use serde::Deserialize;
use std::collections::HashMap;


#[derive(Debug, Deserialize)]
pub struct TournamentFile {
    pub tournament: Tournament,
    pub legends: HashMap<String, u32>,
}


#[derive(Debug, Deserialize)]
pub struct Tournament {
    pub name: String,
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub players: u32,
}


#[derive(Debug, Deserialize, Clone)]
pub struct Legend {
    pub name: String,
    pub title: String,
    pub image: String,
    pub faction: String,
}
