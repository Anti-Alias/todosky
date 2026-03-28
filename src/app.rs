use crate::{GraphError, Task, TaskGraph, TaskId, TaskResponse};
use std::{collections::VecDeque, ops::Range};
use eframe::{App, CreationContext};
use rand::RngExt;
use egui::{
    pos2, vec2, CentralPanel, Color32, Grid, Id, MenuBar, Painter, Panel, Pos2, Rangef, Rect, Scene, ScrollArea, Stroke, Ui, ViewportCommand
};

const EPSILON: f32 = 0.001;
const ARROW_SIDE_LEN: f32           = 10.0;
const CROSS_COLOR: Color32          = Color32::from_rgba_unmultiplied_const(255, 255, 255, 50);
const CROSS_HALF_SIZE: f32          = 200.0;
const CROSS_STROKE: Stroke          = Stroke { width: 1.0, color: CROSS_COLOR };
const DEP_STROKE: Stroke            = Stroke { width: 1.0, color: Color32::WHITE };
const POS_OFFSET_RANGE: Range<f32>  = -40.0..40.0;
const ZOOM_RANGE: Rangef            = Rangef { min: 0.1, max: 1.0 };


pub struct TodoskyApp {
    tasks: TaskGraph,
    scene_rect: Rect,
}

impl App for TodoskyApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let mut actions = VecDeque::new();
        self.show_top_panel(ui, &mut actions);
        self.show_right_panel(ui, &mut actions);
        self.show_central_panel(ui, &mut actions);
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
        Panel::top("top_panel")
            .min_size(32.0)
            .show_inside(ui, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    MenuBar::new().ui(ui, |ui| {
                        ui.menu_button("File", |ui| {
                            if ui.button("Quit").clicked() {
                                actions.push_back(AppAction::Quit);
                            }
                        });
                    });
                });
            });
    }

    /// Center panel, which includes draggable tasks
    fn show_central_panel(&mut self, ui: &mut Ui, actions: &mut VecDeque<AppAction>) {
        CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Task Graph");
                Scene::new()
                    .zoom_range(ZOOM_RANGE)
                    .show(ui, &mut self.scene_rect, |ui| {
                        Self::paint_cross(ui.painter());
                        Self::paint_free_arrows(&self.tasks, ui.painter());
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

    fn paint_cross(painter: &Painter) {
        let left    = pos2(-CROSS_HALF_SIZE, 0.0);
        let right   = pos2(CROSS_HALF_SIZE, 0.0);
        let top     = pos2(0.0, -CROSS_HALF_SIZE);
        let bottom  = pos2(0.0, CROSS_HALF_SIZE);
        painter.line_segment([left, right], CROSS_STROKE);
        painter.line_segment([top, bottom], CROSS_STROKE);
    }

    /// Paints line coming out of task that has no outgoing connection
    fn paint_free_arrows(tasks: &TaskGraph, painter: &Painter) {
        for (_, task) in tasks.iter() {
            let task_center = task.rect().center();
            if let Some(arrow_pos) = task.arrow_pos {
                Self::paint_arrow(task_center, arrow_pos, painter);
            }
        }
    }

    /// Paints lines that connect tasks in the center pane
    fn paint_dependency_arrows(tasks: &TaskGraph, painter: &Painter) {
        for (task_id, task_deps) in tasks.dependencies() {
            let task = tasks.get(task_id).unwrap();
            for dep_task_id in task_deps.iter().copied() {
                let dep_task = tasks.get(dep_task_id).unwrap();
                Self::paint_arrow_between_tasks(task, dep_task, painter);
            }
        }
    }

    fn paint_arrow_between_tasks(parent: &Task, child: &Task, painter: &Painter) {
        if child.pos.distance_sq(parent.pos) < EPSILON*EPSILON { return }
        let child_rect = child.rect();
        let ray_dir = (parent.pos - child.pos).normalized();
        let child_intersection = child_rect.intersects_ray_from_center(ray_dir);
        Self::paint_arrow(parent.pos, child_intersection, painter);
    }

    fn paint_arrow(start: Pos2, end: Pos2, painter: &Painter) {
        if start.distance_sq(end) < EPSILON*EPSILON {
            return; 
        }
        let dir_forwards    = (end - start).normalized() * ARROW_SIDE_LEN;
        let dir_left        = dir_forwards.rot90();
        let tri_back        = end - dir_forwards;
        let tri_left        = tri_back + dir_left;
        let tri_right       = tri_back - dir_left;
        painter.line_segment([start, end], DEP_STROKE);             // Base of line
        painter.line_segment([tri_left, tri_right], DEP_STROKE);    // Triangle
        painter.line_segment([tri_right, end], DEP_STROKE);         // Triangle
        painter.line_segment([end, tri_left], DEP_STROKE);          // Triangle
    }

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
            AppAction::DisplayToast(message)                    => { println!("Toast: {message}"); },
            AppAction::Quit                                     => { ui.send_viewport_cmd(ViewportCommand::Close); }
        }
    }

    fn handle_link_unlink_task(&mut self, parent_id: TaskId, child_pos: Pos2, actions: &mut VecDeque<AppAction>) {
        let Some((child_id, _)) = self.tasks.get_at_pos(child_pos) else { return };
        if !self.tasks.contains_dependency(parent_id, child_id) {
            let result = self.tasks.add_dependency(parent_id, child_id);
            match result {
                Err(GraphError::CycleDetected) => actions.push_back(AppAction::DisplayToast("Cycle detected".to_string())),
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
            rand::rng().random_range(POS_OFFSET_RANGE),
            rand::rng().random_range(POS_OFFSET_RANGE)
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
