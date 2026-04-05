use std::path::Path;
use egui::Rect;
use serde::{Deserialize, Serialize};
use crate::TaskGraph;

/// Fields of app that are serializable
#[derive(Serialize, Deserialize)]
pub struct AppState {
    pub tasks: TaskGraph,
    pub scene_rect: Rect,
}

impl AppState {
    pub fn load_from_file(file: &Path) -> anyhow::Result<AppState> {
        let yaml = std::fs::read_to_string(file)?;
        let state = serde_yaml::from_str(&yaml)?;
        Ok(state)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            tasks: TaskGraph::default(),
            scene_rect: Rect::ZERO,
        }
    }
}


