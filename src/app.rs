use crate::{paint, GraphError, Task, TaskGraph, TaskId, TaskResponse, Toast, ToastId};
use std::sync::mpsc;
use std::{ops::Range, sync::mpsc::{Receiver, Sender, TryRecvError}, time::Duration};
use eframe::{App, CreationContext};
use rand::RngExt;
use egui::{
    vec2, CentralPanel, Color32, Context, Grid, Id, MenuBar, Painter, Panel, Pos2, Rangef, Rect, Scene, Stroke, Ui, ViewportCommand
};

const LINE_STROKE: Stroke           = Stroke { width: 1.0, color: Color32::WHITE };
const TASK_OFFSET_RANGE: Range<f32> = -40.0..40.0;
const ZOOM_RANGE: Rangef            = Rangef { min: 0.1, max: 1.0 };
const COL_WIDTH_TASK_NAME: f32      = 200.0;
const TOAST_DURATION: Duration      = Duration::from_secs(5);
const TOAST_BAR_HEIGHT: f32         = 20.0;

pub struct TodoskyApp {
    tasks: TaskGraph,
    scene_rect: Rect,
    toast: Option<(ToastId, Toast)>,
    sender: Sender<AppAction>,
    receiver: Receiver<AppAction>,
    id_sequence: u32,
}

impl App for TodoskyApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        // Renders UI
        self.show_top_panel(ui);
        self.show_right_panel(ui);
        self.show_central_panel(ui);
        // Handles enqueued actions
        loop {
            match self.receiver.try_recv() {
                Ok(action) => self.handle_action(action, ui.ctx()),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    panic!("Channel disconnected");
                }
            }
        }
    }
}

impl TodoskyApp {

    pub fn new(_ctx: &CreationContext) -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            tasks: TaskGraph::default(),
            scene_rect: Rect::ZERO,
            toast: None,
            sender,
            receiver,
            id_sequence: 0,
        }
    }

    /// Center of the viewport in which new tasks in the graph will be created.
    fn viewport_center(&self) -> Pos2 {
        Pos2::ZERO
    }

    /// Generates an ID from a sequence
    fn gen_id(&mut self) -> u32 {
        let id = self.id_sequence;
        self.id_sequence += 1;
        id
    }

    /// Top panel, which includes the menu bar (File, Edit, Help etc)
    fn show_top_panel(&self, ui: &mut Ui) {
        Panel::top("top_panel").show_inside(ui, |ui| {
            MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Save (Ctrl+S)").clicked() {
                        let toast = Toast::success("Saved");
                        self.sender.send(AppAction::DisplayToast(toast)).unwrap();
                    }
                    if ui.button("Quit (Alt+F4)").clicked() {
                        self.sender.send(AppAction::Quit).unwrap();
                    }
                });
                ui.menu_button("Settings", |ui| {
                    if ui.button("Setting 1").clicked() {
                        let toast = Toast::success("Clicked setting");
                        self.sender.send(AppAction::DisplayToast(toast)).unwrap();
                    }
                    if ui.button("Setting 2").clicked() {
                        let toast = Toast::success("Clicked setting");
                        self.sender.send(AppAction::DisplayToast(toast)).unwrap();
                    }
                    if ui.button("Setting 3").clicked() {
                        let toast = Toast::success("Clicked setting");
                        self.sender.send(AppAction::DisplayToast(toast)).unwrap();
                    }
                });
            });
        });
    }

    /// Center panel, which includes draggable tasks
    fn show_central_panel(&mut self, ui: &mut Ui) {
        CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                self.show_toast_bar(ui);
                Scene::new().zoom_range(ZOOM_RANGE).show(ui, &mut self.scene_rect, |ui| {
                    paint::axis(ui.painter());
                    paint::free_arrows(&self.tasks, ui.painter(), LINE_STROKE);
                    Self::paint_dependency_arrows(&self.tasks, ui.painter());
                    for (task_id, task) in self.tasks.iter_mut() {
                        let response = task.show_as_node(Id::new(task_id), ui);
                        if let TaskResponse::ArrowReleased { release_pos } = response.inner {
                            let action = AppAction::LinkUnlinkTask { task_id, release_pos };
                            self.sender.send(action).unwrap();
                        }
                    }
                });
            });
        });
    }

    fn show_toast_bar(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.set_height(TOAST_BAR_HEIGHT);
            if let Some((_, toast)) = &self.toast {
                toast.show(ui);
            }
        });
    }

    /// Right panel, which shows details about the currently selected task, if any
    fn show_right_panel(&mut self, ui: &mut Ui) {
        Panel::right("right_panel").show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Todo");
                ui.vertical(|ui| {
                    self.show_right_panel_body(ui);
                });
            });
        });
    }

    fn show_right_panel_body(&mut self, ui: &mut Ui) {
        // Top "add task" button
        if ui.button("Add Task").clicked() {
            self.sender.send(AppAction::AddTask).unwrap();
        }
        // Tasks in vertical list
        Grid::new("vertical_task_list").min_col_width(COL_WIDTH_TASK_NAME).show(ui, |ui| {
            for (task_id, task) in self.tasks.iter_mut() {
                let deleted = task.show_as_row(ui);
                ui.end_row();
                if deleted {
                    self.sender.send(AppAction::RemoveTask(task_id)).unwrap();
                }
            }
        });
    }

    /// Painta lines that connect tasks in the center pane.
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
    fn handle_action(&mut self, action: AppAction, ctx: &Context) {
        match action {
            AppAction::AddTask                                  => { self.handle_add_task(); }
            AppAction::RemoveTask(task_id)                      => { self.tasks.remove(task_id); }
            AppAction::LinkUnlinkTask { task_id, release_pos }  => { self.handle_link_unlink_task(task_id, release_pos); },
            AppAction::DisplayToast(toast)                      => { self.handle_display_toast(toast, ctx.clone()) },
            AppAction::RemoveToast(toast_id)                    => { self.handle_remove_toast(toast_id) },
            AppAction::Quit                                     => { ctx.send_viewport_cmd(ViewportCommand::Close); }
        }
    }

    fn handle_remove_toast(&mut self, toast_id: ToastId) {
        if let Some((tid, _)) = self.toast {
            if toast_id == tid {
                self.toast = None;
            }
        }
    }

    /// If a free arrow touches a task, it either adds a dependency or removes it.
    /// It may do nothing it adding the dependency would introduce a cycle.
    fn handle_link_unlink_task(&mut self, parent_id: TaskId, child_pos: Pos2) {
        let Some((child_id, _)) = self.tasks.get_at_pos(child_pos) else { return };
        if !self.tasks.contains_dependency(parent_id, child_id) {
            let result = self.tasks.add_dependency(parent_id, child_id);
            match result {
                Err(GraphError::CycleDetected) => {
                    let toast = Toast::error("Cycle detected");
                    self.sender.send(AppAction::DisplayToast(toast)).unwrap();
                },
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

    fn handle_display_toast(&mut self, toast: Toast, ctx: Context) {
        let toast_id = self.gen_id();
        self.toast = Some((toast_id, toast));
        let sender = self.sender.clone();
        tokio::spawn(async move {
            tokio::time::sleep(TOAST_DURATION).await;
            ctx.request_repaint(); 
            sender.send(AppAction::RemoveToast(toast_id)).unwrap();
        });
    }
}


#[derive(Debug)]
pub enum AppAction {
    Quit,
    AddTask,
    RemoveTask(TaskId),
    LinkUnlinkTask { task_id: TaskId, release_pos: Pos2 },
    DisplayToast(Toast),
    RemoveToast(ToastId),
}

pub type ActionQueue = std::collections::VecDeque<AppAction>;

