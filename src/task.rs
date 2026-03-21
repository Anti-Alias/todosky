use std::collections::HashMap;

use egui::{Color32, Frame, Label, Layout, Pos2, Rect, Response, Sense, Stroke, Ui, UiBuilder, Vec2, Widget};
use slotmap::{new_key_type, SlotMap};

new_key_type! { pub struct TaskId; }

const TASK_SIZE: Vec2 = Vec2::new(150.0, 30.0);
const TASK_STROKE: Stroke = Stroke {
    width: 1.0,
    color: Color32::WHITE,
};

/// Is a Task
#[derive(Clone, PartialEq, Debug)]
pub struct Task {
    pub name: String,
    pub description: Option<String>,
    pub x: f32,
    pub y: f32,
}

impl Task {

    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            x: 0.0,
            y: 0.0,
        }
    }

    pub fn rect(&self) -> Rect {
        Rect {
            min: Pos2::new(self.x, self.y),
            max: Pos2::new(self.x+TASK_SIZE.x, self.y+TASK_SIZE.y),
        }
    }

    pub fn show(&mut self, ui: &mut Ui) -> Response {
        let rect = self.rect();
        let builder = UiBuilder::default().sense(Sense::DRAG).max_rect(rect);
        let response = ui.scope_builder(builder, |ui| {
            Frame::NONE
                .stroke(TASK_STROKE)
                .inner_margin(5)
                .fill(ui.visuals().window_fill)
                .corner_radius(3)
                .show(ui, |ui| {
                    self.show_content(ui);
                });
        }).response;
        self.x += response.drag_delta().x;
        self.y += response.drag_delta().y;
        response
    }

    fn show_content(&self, ui: &mut Ui) {
        ui.set_min_size(TASK_SIZE);
        ui.set_max_size(TASK_SIZE);
        ui.with_layout(Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
            ui.add(Label::new(&self.name).selectable(false));
        });
    }
}

impl Widget for &Task {
    fn ui(self, _ui: &mut Ui) -> Response {
        todo!()
    }
}


pub struct TaskGraph {
    tasks: SlotMap<TaskId, Task>,
    dependencies: HashMap<TaskId, Vec<TaskId>>,
}

impl TaskGraph {

    pub fn insert(&mut self, task: Task) -> TaskId {
        self.tasks.insert(task)
    }

    pub fn add_dependency(&mut self, task_id_a: TaskId, task_id_b: TaskId) -> bool {
        if !self.tasks.contains_key(task_id_a) || !self.tasks.contains_key(task_id_b) {
            return false
        }
        let task_deps = self.dependencies
            .entry(task_id_a)
            .or_default();
        task_deps.push(task_id_b);
        true
    }

    pub fn get(&self, id: TaskId) -> Option<&Task> {
        self.tasks.get(id)
    }

    pub fn get_mut(&mut self, id: TaskId) -> Option<&mut Task> {
        self.tasks.get_mut(id)
    }

    pub fn iter(&self) -> impl Iterator<Item=(TaskId, &Task)> {
        self.tasks.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item=(TaskId, &mut Task)> {
        self.tasks.iter_mut()
    }

    pub fn dependencies(&self) -> impl Iterator<Item=(TaskId, &[TaskId])> {
        self.dependencies.iter()
            .map(|(task_id, deps)| (*task_id, deps.as_slice()))
    }
}

impl Default for TaskGraph {
    fn default() -> Self {
        Self {
            tasks: SlotMap::default(),
            dependencies: HashMap::default(), 
        }
    }
}

