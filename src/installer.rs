use std::{path::PathBuf, str::FromStr};


use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Repo {
    pub artifacts: Vec<RepoArtifact>,
    pub changelog: String,
    pub date: String,
    pub hash: String,
    pub id: String,
    pub short_hash: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct RepoArtifact {
    pub description: String,
    pub file: String,
}

#[derive(Clone, Debug)]
pub struct ArtifactDetails {
    pub game_type: GameType,
    pub version: String,
    pub build_id: String,
    pub arch: String,
    pub file_name: String,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum GameType {
    UnityMono,
    UnityIL2CPP,
    NetLauncher,
}

impl GameType {
    pub fn hint(&self) -> String {
        match self {
            GameType::UnityMono => "Based on 'Managed' folder inside of _Data folder".to_owned(),
            GameType::UnityIL2CPP => {
                "Based on 'il2cpp_data' folder inside of _Data folder".to_owned()
            }
            GameType::NetLauncher => {
                "Based on the fact nothing mentioned above didn't fit".to_owned()
            }
        }
    }
}

impl ToString for GameType {
    fn to_string(&self) -> String {
        match self {
            GameType::UnityMono => "UnityMono".to_owned(),
            GameType::UnityIL2CPP => "UnityIL2CPP".to_owned(),
            GameType::NetLauncher => "NetLauncher".to_owned(),
        }
    }
}

impl FromStr for GameType {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "UnityMono" => Ok(GameType::UnityMono),
            "UnityIL2CPP" => Ok(GameType::UnityIL2CPP),
            "NetLauncher" => Ok(GameType::NetLauncher),
            _ => Err(Error::invalid_game_type()),
        }
    }
}

#[derive(Default)]
pub struct Installer {
    pub repo: Repo,
    pub artifacts: Vec<ArtifactDetails>,
    pub game_folder: Option<PathBuf>,
    pub game_type: Option<GameType>,
    pub installed: bool,
    pub game_selected: bool,
    pub error: Option<Error>,
}
