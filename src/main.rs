use eframe::NativeOptions;
use egui::ViewportBuilder;
use todosky::TodoskyApp;

fn main() -> eframe::Result {
    env_logger::init();
    let viewport = ViewportBuilder::default()
        .with_inner_size([400.0, 300.0])
        .with_min_inner_size([300.0, 220.0]);
    let native_options = NativeOptions { viewport, ..Default::default() };
    eframe::run_native(
        "Todosky",
        native_options,
        Box::new(|cc| Ok(Box::new(TodoskyApp::new(cc)))),
    )
}
