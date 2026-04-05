use std::path::Path;
use eframe::NativeOptions;
use egui::ViewportBuilder;
use todosky::{paths::Paths, AppSettings, AppState, TodoskyApp};
use anyhow::Result;

const APP_NAME: &str = "Todosky";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let paths = Paths::new()?;
    let app_data = load_app_data(&paths);
    let viewport = ViewportBuilder::default();
    let native_options = NativeOptions { viewport, ..Default::default() };
    eframe::run_native(
        APP_NAME,
        native_options,
        Box::new(move |cc| Ok(Box::new(TodoskyApp::new(
            cc,
            app_data.state,
            app_data.settings,
            paths
        )))),
    )?;
    Ok(())
}

/// Loads both application settings and state from files
fn load_app_data(paths: &Paths) -> AppData {
    let mut settings = load_settings(paths);
    let state = match &settings.current_file {
        Some(current_file) => match load_state_from_file(current_file) {
            Ok(state) => state,
            Err(err) => {
                log::error!("Failed to load state file: {err}");
                settings.current_file = None;
                AppState::default()
            }
        }
        None => AppState::default(),
    };
    AppData { settings, state }
}

fn load_state_from_file(file: &Path) -> Result<AppState> {
    let yaml = std::fs::read_to_string(file)?;
    let state = serde_yaml::from_str(&yaml)?;
    Ok(state)
}

fn load_settings(paths: &Paths) -> AppSettings {
    let yaml = match std::fs::read_to_string(&paths.settings_file) {
        Ok(yaml) => yaml,
        Err(_) => return AppSettings::default(),
    };
    match serde_yaml::from_str(&yaml) {
        Ok(settings) => settings,
        Err(err) => {
            log::error!("Failed to load settings file: {err}");
            AppSettings::default()
        }
    }
}

struct AppData {
    state: AppState,
    settings: AppSettings,
}
