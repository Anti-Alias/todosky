use crate::{GraphError, Task, TaskGraph, TaskId, TaskResponse, paint};
use std::{collections::VecDeque, ops::Range};
use eframe::{App, CreationContext};
use rand::RngExt;
use egui::{
    vec2, CentralPanel, Color32, Grid, Id, MenuBar, Painter, Panel, Pos2, Rangef, Rect, Scene, Stroke, Ui, ViewportCommand
};

const LINE_STROKE: Stroke           = Stroke { width: 1.0, color: Color32::WHITE };
const TASK_OFFSET_RANGE: Range<f32> = -40.0..40.0;
const ZOOM_RANGE: Rangef            = Rangef { min: 0.1, max: 1.0 };

pub struct TodoskyApp {
    tasks: TaskGraph,
    scene_rect: Rect,
}

impl App for TodoskyApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        // Renders UI
        let mut actions = VecDeque::new();
        self.show_top_panel(ui, &mut actions);
        self.show_right_panel(ui, &mut actions);
        self.show_central_panel(ui, &mut actions);
        // Handles enqueued actions
        while let Some(action) = actions.pop_front() {
            self.handle_action(action, &mut actions, ui);
        }
    }
}

impl TodoskyApp {

    pub fn new(_ctx: &CreationContext) -> Self {
        Self {
            tasks: TaskGraph::default(),
            scene_rect: Rect::ZERO,
        }
    }

    /// Center of the viewport in which new tasks in the graph will be created.
    fn viewport_center(&self) -> Pos2 {
        Pos2::ZERO
    }

    /// Top panel, which includes the menu bar (File, Edit, Help etc)
    fn show_top_panel(&self, ui: &mut Ui, actions: &mut VecDeque<AppAction>) {
        Panel::top("top_panel").show_inside(ui, |ui| {
            MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Save (Ctrl+S)").clicked() {
                        actions.push_back(AppAction::display_toast("Saved"));
                    }
                    if ui.button("Quit (Alt+F4)").clicked() {
                        actions.push_back(AppAction::Quit);
                    }
                });
                ui.menu_button("Settings", |ui| {
                    if ui.button("Setting 1").clicked() {
                        actions.push_back(AppAction::display_toast("Clicked setting"));
                    }
                    if ui.button("Setting 2").clicked() {
                        actions.push_back(AppAction::display_toast("Clicked setting"));
                    }
                    if ui.button("Setting 3").clicked() {
                        actions.push_back(AppAction::display_toast("Clicked setting"));
                    }
                });
            });
        });
    }

    /// Center panel, which includes draggable tasks
    fn show_central_panel(&mut self, ui: &mut Ui, actions: &mut VecDeque<AppAction>) {
        CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                Scene::new().zoom_range(ZOOM_RANGE).show(ui, &mut self.scene_rect, |ui| {
                    paint::axis(ui.painter());
                    paint::free_arrows(&self.tasks, ui.painter(), LINE_STROKE);
                    Self::paint_dependency_arrows(&self.tasks, ui.painter());
                    for (task_id, task) in self.tasks.iter_mut() {
                        let response = task.show_as_node(Id::new(task_id), ui);
                        if let TaskResponse::ArrowReleased { release_pos } = response.inner {
                            actions.push_back(AppAction::LinkUnlinkTask { task_id, release_pos });
                        }
                    }
                });
            });
        });
    }

    /// Right panel, which shows details about the currently selected task, if any
    fn show_right_panel(&mut self, ui: &mut Ui, actions: &mut VecDeque<AppAction>) {
        Panel::right("right_panel").show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Todo");
                ui.vertical(|ui| {
                    self.show_right_panel_body(ui, actions);
                });
            });
        });
    }

    fn show_right_panel_body(&mut self, ui: &mut Ui, actions: &mut VecDeque<AppAction>) {
        // Top "add task" button
        if ui.button("Add Task").clicked() {
            actions.push_back(AppAction::AddTask);
        }
        // Tasks in vertical list
        Grid::new("vertical_task_list").show(ui, |ui| {
            for (task_id, task) in self.tasks.iter_mut() {
                let deleted = task.show_as_row(ui);
                ui.end_row();
                if deleted {
                    actions.push_back(AppAction::RemoveTask(task_id));
                }
            }
        });
    }

    /// Paints lines that connect tasks in the center pane.
    fn paint_dependency_arrows(tasks: &TaskGraph, painter: &Painter) {
        for (task_id, task_deps) in tasks.dependencies() {
            let task = tasks.get(task_id).unwrap();
            for dep_task_id in task_deps.iter().copied() {
                let dep_task = tasks.get(dep_task_id).unwrap();
                paint::arrow_between_rects(task.rect(), dep_task.rect(), painter, LINE_STROKE);
            }
        }
    }

    /// Handles an enqueued action.
    /// May fire more actions to be handled.
    fn handle_action(
        &mut self,
        action: AppAction,
        actions: &mut VecDeque<AppAction>,
        ui: &mut Ui,
    ) {
        match action {
            AppAction::AddTask                                  => { self.handle_add_task(); }
            AppAction::RemoveTask(task_id)                      => { self.tasks.remove(task_id); }
            AppAction::LinkUnlinkTask { task_id, release_pos }  => { self.handle_link_unlink_task(task_id, release_pos, actions); },
            AppAction::DisplayToast(message)                    => { println!("Toast: {message}"); },   // TODO: Improve
            AppAction::Quit                                     => { ui.send_viewport_cmd(ViewportCommand::Close); }
        }
    }

    /// If a free arrow touches a task, it either adds a dependency or removes it.
    /// It may do nothing it adding the dependency would introduce a cycle.
    fn handle_link_unlink_task(
        &mut self,
        parent_id: TaskId,
        child_pos: Pos2,
        actions: &mut VecDeque<AppAction>,
    ) {
        let Some((child_id, _)) = self.tasks.get_at_pos(child_pos) else { return };
        if !self.tasks.contains_dependency(parent_id, child_id) {
            let result = self.tasks.add_dependency(parent_id, child_id);
            match result {
                Err(GraphError::CycleDetected) => actions.push_back(AppAction::display_toast("Cycle detected")),
                _ => {},
            }
        }
        else {
            self.tasks.remove_dependency(parent_id, child_id);
        }
    }

    /// Adds a new task.
    /// Invoked when "Add Task" button is pressed.
    fn handle_add_task(&mut self) {
        let mut task = Task::new("New Task");
        let offset = vec2(
            rand::rng().random_range(TASK_OFFSET_RANGE),
            rand::rng().random_range(TASK_OFFSET_RANGE)
        );
        task.pos = self.viewport_center() + offset;
        self.tasks.insert(task);
    }
}


#[derive(Debug)]
pub enum AppAction {
    Quit,
    AddTask,
    RemoveTask(TaskId),
    LinkUnlinkTask { task_id: TaskId, release_pos: Pos2 },
    DisplayToast(String),
}

impl AppAction {
    pub fn display_toast(message: impl Into<String>) -> Self {
        Self::DisplayToast(message.into())
    }
}
