#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod bepinex;
pub mod installer;

use bepinex::BepInEx;
use eframe::{run_native, NativeOptions};
use installer::Game;
use std::{
    error,
    path::{Path, PathBuf},
};
use steamlocate::SteamDir;

use crate::installer::Installer;

const OLDEST_SUPPORTED_STABLE: &str = "v5.4.11";
const OLDEST_SUPPORTED_BE: u16 = 510;

#[tokio::main]
async fn main() {
    let octocrab = octocrab::instance();

    let _stable_releases = BepInEx::get_stable_releases(octocrab).await.unwrap();

    let games = get_unity_games();
    if games.is_err() {
        return;
    }
    let games = games.unwrap();

    let options = NativeOptions {
        follow_system_theme: true,
        transparent: false,
        resizable: false,
        initial_window_size: Some(eframe::egui::vec2(400.0, 450.0)),
        ..NativeOptions::default()
    };

    let bepinex = BepInEx {
        releases: _stable_releases,
    };
    let installer = Installer {
        bepinex,
        games,
        ..Installer::default()
    };

    run_native(
        "BepInEx Installer",
        options,
        Box::new(|_cc| Box::new(installer)),
    )
}

fn get_unity_games() -> Result<Vec<Game>, Box<dyn error::Error>> {
    let mut unity_games: Vec<Game> = Vec::new();

    let mut steamapps = SteamDir::locate().unwrap_or_default();
    let apps = steamapps.apps();

    apps.iter().for_each(|(_id, app)| {
        if app.is_none() {
            return;
        }
        let app = app.as_ref().unwrap();
        let path = Path::new(&app.path);
        if path.join("UnityPlayer.dll").exists() {
            unity_games.push(Game::new(
                app.name.clone().unwrap_or_default(),
                "a".to_owned(),
                app.path.to_owned(),
            ));
        }
    });
    Ok(unity_games)
}

fn get_dll_version(path: PathBuf) -> Result<String, Box<dyn error::Error>> {
    let file = pelite::FileMap::open(path.as_path())?;
    let img = pelite::PeFile::from_bytes(file.as_ref())?;
    let resources = img.resources()?;
    let version_info = resources.version_info()?;
    let lang = version_info
        .translation()
        .get(0)
        .ok_or("Failed to get lang")?;
    let strings = version_info.file_info().strings;
    let string = strings
        .get(lang)
        .ok_or("Failed to get strings for that lang")?;
    let version = string
        .get("ProductVersion")
        .ok_or("Failed to get prod. version")?;

    Ok(version.to_owned())
}

fn get_installed_bepinex_version(game: &Game) -> Option<String> {
    let core_path = game.path.join("BepInEx").join("core");
    if !core_path.exists() {
        return None;
    }

    if core_path.join("BepInEx.Core.dll").exists() {
        return get_dll_version(core_path.join("BepInEx.Core.dll")).ok();
    } else if core_path.join("BepInEx.dll").exists() {
        return get_dll_version(core_path.join("BepInEx.dll")).ok();
    }
    None
}
