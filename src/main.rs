use eframe::NativeOptions;
use egui::ViewportBuilder;
use todosky::{paths::Paths, TodoskyApp};
use anyhow::Result;

const APP_NAME: &str = "Todosky";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let paths = Paths::new()?;
    let viewport = ViewportBuilder::default();
    let native_options = NativeOptions { viewport, ..Default::default() };
    eframe::run_native(
        APP_NAME,
        native_options,
        Box::new(move |_cc| Ok(Box::new(TodoskyApp::load(paths)))),
    )?;
    Ok(())
}

