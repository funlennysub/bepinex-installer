use std::{fmt::Display, path::PathBuf};

use eframe::{
    egui::{CentralPanel, ComboBox, Ui},
    epaint::{vec2, Vec2},
    App,
};

use crate::{
    bepinex::{BepInEx, BepInExRelease},
    get_installed_bepinex_version,
};

#[derive(Default)]
pub struct Installer {
    pub settings: bool,
    pub advanced_mode: bool,
    pub advanced_settings: Option<AdvancedSettings>,
    pub bepinex: BepInEx,
    pub selected_bix: Option<BepInExRelease>,
    pub games: Vec<Game>,
    pub selected_game: Option<Game>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancedSettings {
    picker: bool,
    bleeding_edge: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    pub name: String,
    pub arch: String,
    pub path: PathBuf,
}

impl Game {
    pub fn new(name: String, arch: String, path: PathBuf) -> Self {
        Self { name, arch, path }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self {
            name: "Not selected".to_owned(),
            arch: "x86".to_owned(),
            path: Default::default(),
        }
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Installer {
    fn show_games_select(self: &mut Installer, ui: &mut Ui) {
        ComboBox::from_id_source("game_selector")
            .width(ui.available_width() - 8.0)
            .selected_text(
                self.selected_game
                    .as_ref()
                    .map(|e| format!("{}", e))
                    .unwrap_or_else(|| "Select a game".to_owned()),
            )
            .show_ui(ui, |ui| {
                for game in self.games.iter() {
                    ui.selectable_value(&mut self.selected_game, Some(game.to_owned()), &game.name);
                }
            });
    }

    fn show_bix_select(self: &mut Installer, ui: &mut Ui) {
        ComboBox::from_id_source("bix_selector")
            .width(ui.available_width() - 8.0)
            .selected_text(
                self.selected_bix
                    .as_ref()
                    .map(|e| format!("{}", e))
                    .unwrap_or_else(|| "Select BepInEx version".to_owned()),
            )
            .show_ui(ui, |ui| {
                for bix_ver in self.bepinex.releases.iter() {
                    ui.selectable_value(
                        &mut self.selected_bix,
                        Some(bix_ver.to_owned()),
                        &bix_ver.version,
                    );
                }
            });
    }
}

impl App for Installer {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            let prev_game = self.selected_game.clone();
            self.show_games_select(ui);
            if prev_game != self.selected_game && self.selected_game.is_some() {
                let installed_bix =
                    get_installed_bepinex_version(self.selected_game.as_ref().unwrap());
                println!("{:#?}", installed_bix)
            }

            let prev_bix = self.selected_bix.clone();
            self.show_bix_select(ui);
            if prev_bix != self.selected_bix {
                println!("bix changed");
            }

            ui.allocate_space(vec2(ui.available_width(), ui.available_height() - 50.0));

            ui.centered_and_justified(|ui| {
                if ui.button("Install").clicked() {
                    println!("{:?}", self.selected_game);
                    println!("{:?}", self.selected_bix);
                }
            })
        });
    }
}
