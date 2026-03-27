use std::collections::HashMap;

use egui::{Color32, Frame, Id, Label, Layout, PointerButton, Pos2, Rect, Response, Sense, Stroke, Ui, UiBuilder, Vec2, Widget};
use slotmap::{new_key_type, SlotMap};

new_key_type! { pub struct TaskId; }

const TASK_SIZE: Vec2 = Vec2::new(150.0, 30.0);
const TASK_STROKE: Stroke = Stroke {
    width: 1.0,
    color: Color32::WHITE,
};

#[derive(Clone, PartialEq, Debug)]
pub struct Task {
    pub name: String,
    pub description: Option<String>,
    pub pos: Pos2,
    /// When dragging with RMB, determines end point of line to be drawn.
    pub arrow_pos: Option<Pos2>,
}

impl Task {

    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            pos: Pos2::ZERO,
            arrow_pos: None,
        }
    }

    /// Size / location this task occupies
    pub fn rect(&self) -> Rect {
        let half_size = TASK_SIZE / 2.0;
        Rect {
            min: self.pos - half_size,
            max: self.pos + half_size,
        }
    }

    /// Renders task as a "node" in the center pane
    pub fn show_as_node(&mut self, id: Id, ui: &mut Ui) -> Response {

        // Renders task in a draggable area
        let rect = self.rect();
        let builder = UiBuilder::default()
            .id(id)
            .sense(Sense::DRAG)
            .max_rect(rect);
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

        // Handles dragging logic from response
        if response.dragged_by(PointerButton::Primary) {
            self.pos += response.drag_delta();
        }

        // Handles arrow drawing when dragging with right mouse
        if let Some(pointer_pos) = ui.pointer_latest_pos() && response.dragged_by(PointerButton::Secondary) {
            self.arrow_pos = Some(pointer_pos);
        }
        else {
            self.arrow_pos = None;
        }

        // Makes mouse hover if mouse is over the task
        if response.hovered() {
            ui.set_cursor_icon(egui::CursorIcon::Grabbing);
        }
        response
    }

    pub fn show_as_row(&mut self, ui: &mut Ui) -> bool {
        ui.label(&self.name);
        ui.button("x").clicked()
    }

    /// Renders content of task UI
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

    pub fn retain<F>(&mut self, predicate: F)
    where
        F: FnMut(TaskId, &mut Task) -> bool,
    {
        self.tasks.retain(predicate);
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

