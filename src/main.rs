use eframe::NativeOptions;
use egui::ViewportBuilder;
use todosky::{paths::Paths, AppSettings, AppState, TodoskyApp};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // Loads paths
    let paths = Paths::new()?;
    let settings = AppSettings {
        current_file: Some(String::from("thefile.yml")),
    };
    let state = load_app_state(&settings);

    // Runs egui app 
    let viewport = ViewportBuilder::default();
    let native_options = NativeOptions { viewport, ..Default::default() };
    eframe::run_native(
        "Todosky",
        native_options,
        Box::new(move |cc| Ok(Box::new(TodoskyApp::new(cc, state, settings, paths)))),
    )?;
    Ok(())
}

fn load_app_state(settings: &AppSettings) -> AppState {
    match &settings.current_file {
        Some(current_file) => match load_state_from_file(current_file) {
            Ok(state) => state,
            Err(err) => {
                log::error!("Failed to load settings file: {err}");
                AppState::default()
            }
        }
        None => AppState::default(),
    } 
}

fn load_state_from_file(file: &str) -> Result<AppState> {
    let yaml = std::fs::read_to_string(file)?;
    let state = serde_yaml::from_str(&yaml)?;
    Ok(state)
}

