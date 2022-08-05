#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod error;
mod installer;

use std::{fs, io::Cursor, path::PathBuf, str::FromStr};

use eframe::run_native;
use egui::{CentralPanel, Style, Vec2};
use error::Error;
use installer::{ArtifactDetails, GameType, Installer, Repo, RepoArtifact};
use regex::Regex;
use reqwest::StatusCode;
use rfd::FileDialog;
use zip::ZipArchive;

const BEPINEX_ARTIFACTS_URL: &str =
    "https://builds.bepinex.dev/api/projects/bepinex_be/artifacts/latest";

const BASE_BEPINEX_URL: &str = "https://builds.bepinex.dev/projects/bepinex_be";

fn main() -> Result<(), Error> {
    let mut installer = Installer::default();
    let repo = get_repo();
    match repo {
        Ok(repo) => {
            installer.artifacts = repo
                .artifacts
                .iter()
                .filter_map(get_artifact_details)
                .collect();
        }
        Err(e) => installer.error = Some(e),
    }

    run_native(
        "BepInEx Installer",
        eframe::NativeOptions {
            initial_window_size: Some(Vec2::new(900., 400.)),
            ..Default::default()
        },
        Box::new(|cc| {
            cc.egui_ctx.set_style(Style::default());
            Box::new(installer)
        }),
    )
}

impl eframe::App for Installer {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            if self.error.is_some() {
                egui::Window::new("Error occurred").show(ctx, |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
                ui.set_enabled(false);
            }
            ui.horizontal(|ui| {
                if ui.button("Select game").clicked() {
                    if let Some(path) = pick_file() {
                        self.game_folder = Some(path.parent().unwrap().to_path_buf());
                        self.game_type = get_game_type(self.game_folder.as_ref().unwrap())
                    }
                }
                if let Some(game_folder) = &self.game_folder {
                    ui.monospace(game_folder.to_str().unwrap().to_string());
                }
            });
            ui.horizontal(|ui| {
                ui.label("Game type: ");
                if self.game_type.is_none() {
                    ui.monospace("Failed to identify game type");
                } else {
                    ui.monospace(&self.game_type.as_ref().unwrap().to_string())
                        .on_hover_text(&self.game_type.as_ref().unwrap().hint());
                }
            });
            if self.game_type.is_some() && ui.button("Install").clicked() {
                let install = install(
                    self.game_type,
                    self.artifacts.clone(),
                    self.game_folder.clone(),
                );
                match install {
                    Ok(_) => self.installed = true,
                    Err(e) => {
                        println!("{:?}", e);
                        self.error = Some(e)
                    }
                }
            }
            if self.installed {
                egui::Window::new("Tada 🎉").show(ctx, |ui| {
                    ui.label("BepInEx has been successfully installed!");
                    if ui.button("quit").clicked() {
                        frame.quit();
                    }
                });
            }
        });
    }
}

fn pick_file() -> Option<PathBuf> {
    FileDialog::new()
        .add_filter("Game executable", &["exe"])
        .pick_file()
}

fn get_repo() -> Result<Repo, Error> {
    let req = reqwest::blocking::Client::new()
        .get(BEPINEX_ARTIFACTS_URL)
        .send()?;
    if req.status() != StatusCode::OK {
        return Err(Error::http(req.status()));
    }

    let req: Repo = req.json()?;
    Ok(req)
}

fn get_artifact_details(repo: &RepoArtifact) -> Option<ArtifactDetails> {
    let regex = Regex::new(r"BepInEx_(NetLauncher|(?:Unity(?:Mono|IL2CPP)))(?:_(x\d+|unix)){0,1}_(?:[a-zA-Z0-9]{7})_((?:[0-9]){1,}\.(?:[0-9]){1,}\.(?:[0-9]){1,})-be\.(\d+)\.zip").unwrap();
    let captures = regex.captures(&repo.file).unwrap();

    let game_type = captures.get(1)?.as_str().to_string();
    let arch = captures
        .get(2)
        .map(|e| e.as_str())
        .unwrap_or("x64")
        .to_string();
    let version = captures.get(3)?.as_str().to_string();
    let build_id = captures.get(4)?.as_str().to_string();

    Some(ArtifactDetails {
        game_type: GameType::from_str(game_type.as_str()).unwrap(),
        arch,
        version,
        file_name: repo.file.to_string(),
        build_id,
    })
}

fn get_game_type(path: &PathBuf) -> Option<GameType> {
    let mono = "Managed";
    let il2cpp = "il2cpp_data";

    if let Ok(dir) = fs::read_dir(path) {
        let data_dir = dir.filter_map(Result::ok).find(|el| {
            el.file_name().to_str().unwrap().ends_with("_Data") && el.file_type().unwrap().is_dir()
        });

        let data_dir = data_dir.as_ref()?.path();
        if data_dir.join(mono).exists() {
            Some(GameType::UnityMono)
        } else if data_dir.join(il2cpp).exists() {
            Some(GameType::UnityIL2CPP)
        } else {
            Some(GameType::NetLauncher)
        }
    } else {
        None
    }
}

fn get_download_url(artifact: &ArtifactDetails) -> String {
    format!(
        "{}/{}/{}",
        BASE_BEPINEX_URL, artifact.build_id, artifact.file_name
    )
}

fn install(
    game_type: Option<GameType>,
    artifacts: Vec<ArtifactDetails>,
    path: Option<PathBuf>,
) -> Result<(), Error> {
    if game_type.is_none() || path.is_none() {
        return Err(Error::invalid_game_type());
    }

    let game_type = game_type.unwrap();
    let path = path.unwrap();

    let artifact = artifacts
        .iter()
        .find(|artifact| artifact.game_type == game_type && artifact.arch == "x64");

    if artifact.is_none() {
        return Err(Error::invalid_game_type());
    }
    let artifact = artifact.unwrap();
    let url = get_download_url(artifact);

    let resp = reqwest::blocking::Client::new().get(url).send()?;

    if resp.status() != StatusCode::OK {
        return Err(Error::http(resp.status()));
    }

    let bytes = resp.bytes()?;
    let zip = ZipArchive::new(Cursor::new(bytes.to_vec()))
        .unwrap()
        .extract(path);
    if zip.is_err() {
        return Err(Error::zip_error(zip.err().unwrap()));
    }
    Ok(())
}
