use eframe::NativeOptions;
use egui::ViewportBuilder;
use todosky::TodoskyApp;

#[tokio::main]
async fn main() -> eframe::Result {
    env_logger::init();
    let viewport = ViewportBuilder::default();
    let native_options = NativeOptions { viewport, ..Default::default() };
    eframe::run_native(
        "Todosky",
        native_options,
        Box::new(|cc| Ok(Box::new(TodoskyApp::new(cc)))),
    )
}
