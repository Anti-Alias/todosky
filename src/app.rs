use crate::paths::Paths;
use crate::{paint, GraphError, Task, TaskGraph, TaskId, TaskResponse, Toast, ToastId};
use crate::file_utils;
use std::path::PathBuf;
use std::sync::mpsc;
use std::{ops::Range, sync::mpsc::{Receiver, Sender, TryRecvError}, time::Duration};
use eframe::{App, CreationContext};
use rand::RngExt;
use rfd::FileDialog;
use serde::{Serialize, Deserialize};
use egui::{
    vec2, CentralPanel, Color32, Context, Grid, Id, InputState, Key, KeyboardShortcut, MenuBar, Modifiers, Painter, Panel, Pos2, Rangef, Rect, Scene, Stroke, Ui, ViewportCommand
};

// UI constants
const LINE_STROKE: Stroke                   = Stroke { width: 1.0, color: Color32::WHITE };
const TASK_OFFSET_RANGE: Range<f32>         = -40.0..40.0;
const ZOOM_RANGE: Rangef                    = Rangef { min: 0.1, max: 1.0 };
const COL_WIDTH_TASK_NAME: f32              = 200.0;
const TOAST_DURATION: Duration              = Duration::from_secs(5);
const TOAST_BAR_HEIGHT: f32                 = 20.0;
// File Menu
const SAVE_AS_TITLE: &str = "Save As";
const DEFAULT_FILE_NAME: &str = "todo.yml";
const DEFAULT_FILE_EXTENSION: &str = "yml";
// Shortcuts
const SAVE_SHORTCUT: KeyboardShortcut       = KeyboardShortcut::new(Modifiers::CTRL, Key::S);
const SAVE_AS_SHORTCUT: KeyboardShortcut    = KeyboardShortcut::new(Modifiers::CTRL.plus(Modifiers::SHIFT), Key::S);

pub struct TodoskyApp {
    state: AppState,        // State that is serializable to a specified file
    settings: AppSettings,  // Global settings that is saved to its own file
    // Non-serializable fields
    paths: Paths,
    id_sequence: u32,
    toast: Option<(ToastId, Toast)>,
    channel: Channel,
}

/// Fields of app that are serializable
#[derive(Serialize, Deserialize)]
pub struct AppState {
    tasks: TaskGraph,
    scene_rect: Rect,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            tasks: TaskGraph::default(),
            scene_rect: Rect::ZERO,
        }
    }
}

/// Saved to global settings file
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AppSettings {
    pub current_file: Option<PathBuf>,
}

impl App for TodoskyApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        // Handles keyboard shortcuts
        ui.input_mut(|input| {
            self.handle_shortcuts(input);
        });
        // Renders UI
        self.show_top_panel(ui);
        self.show_right_panel(ui);
        self.show_central_panel(ui);
        // Handles enqueued actions
        loop {
            match self.channel.receiver.try_recv() {
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

    pub fn new(_ctx: &CreationContext, state: AppState, settings: AppSettings, paths: Paths) -> Self {
        Self {
            state,
            settings,
            id_sequence: 0,
            toast: None,
            channel: Channel::default(),
            paths,
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

    fn handle_shortcuts(&mut self, input: &mut InputState) {
        if input.consume_shortcut(&SAVE_AS_SHORTCUT) {
            self.channel.send(AppAction::SaveAs);
        }
        if input.consume_shortcut(&SAVE_SHORTCUT) {
            self.channel.send(AppAction::Save);
        }
    }

    /// Top panel, which includes the menu bar (File, Edit, Help etc)
    fn show_top_panel(&self, ui: &mut Ui) {
        Panel::top("top_panel").show_inside(ui, |ui| {
            MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Save (Ctrl+S)").clicked() {
                        self.channel.send(AppAction::Save);
                    }
                    if ui.button("Save As (Ctrl+Shift+S)").clicked() {
                        self.channel.send(AppAction::SaveAs);
                    }
                    if ui.button("Quit (Alt+F4)").clicked() {
                        self.channel.send(AppAction::Quit);
                    }
                });
                ui.menu_button("Settings", |ui| {
                    if ui.button("Setting 1").clicked() {
                        let toast = Toast::success("Clicked setting");
                        self.channel.send(AppAction::DisplayToast(toast));
                    }
                    if ui.button("Setting 2").clicked() {
                        let toast = Toast::success("Clicked setting");
                        self.channel.send(AppAction::DisplayToast(toast));
                    }
                    if ui.button("Setting 3").clicked() {
                        let toast = Toast::success("Clicked setting");
                        self.channel.send(AppAction::DisplayToast(toast));
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
                Scene::new().zoom_range(ZOOM_RANGE).show(ui, &mut self.state.scene_rect, |ui| {
                    paint::axis(ui.painter());
                    paint::free_arrows(&self.state.tasks, ui.painter(), LINE_STROKE);
                    Self::paint_dependency_arrows(&self.state.tasks, ui.painter());
                    for (task_id, node) in self.state.tasks.iter_mut() {
                        let response = node.task.show_as_node(Id::new(task_id), ui);
                        if let TaskResponse::ArrowReleased { release_pos } = response.inner {
                            let action = AppAction::LinkUnlinkTask { task_id, release_pos };
                            self.channel.send(action);
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
                    self.show_todo_list(ui);
                });
                if self.has_backlog_tasks() {
                    ui.heading("Backlog");
                    ui.vertical(|ui| {
                        self.show_backlog(ui);
                    });
                }
            });
        });
    }

    fn show_todo_list(&mut self, ui: &mut Ui) {
        if ui.button("Add Task").clicked() {
            self.channel.send(AppAction::AddTask);
        }
        Grid::new("todo_list").min_col_width(COL_WIDTH_TASK_NAME).show(ui, |ui| {
            for (task_id, node) in self.state.tasks.iter_mut() {
                if !node.children().is_empty() { continue }
                let deleted = node.task.show_as_row(ui);
                ui.end_row();
                if deleted {
                    self.channel.send(AppAction::RemoveTask(task_id));
                }
            }
        });
    }

    fn show_backlog(&mut self, ui: &mut Ui) {
        Grid::new("backlog").min_col_width(COL_WIDTH_TASK_NAME).show(ui, |ui| {
            for (task_id, node) in self.state.tasks.iter_mut() {
                if node.children().is_empty() { continue }
                let deleted = node.task.show_as_row(ui);
                ui.end_row();
                if deleted {
                    self.channel.send(AppAction::RemoveTask(task_id));
                }
            }
        });
    }

    fn has_backlog_tasks(&self) -> bool {
        for (_, node) in self.state.tasks.iter() {
            if !node.children().is_empty() {
                return true;
            }
        }
        false
    }

    /// Paints dependency arrows between tasks  
    fn paint_dependency_arrows(tasks: &TaskGraph, painter: &Painter) {
        for (_, parent) in tasks.iter() {
            for child_id in parent.children().iter().copied() {
                let child = tasks.get(child_id).unwrap();
                paint::arrow_between_rects(parent.task.rect(), child.task.rect(), painter, LINE_STROKE);
            }
        }
    }

    /// Handles an enqueued action.
    fn handle_action(&mut self, action: AppAction, ctx: &Context) {
        match action {
            AppAction::AddTask                                  => { self.handle_add_task(); }
            AppAction::RemoveTask(task_id)                      => { self.state.tasks.remove(task_id); }
            AppAction::LinkUnlinkTask { task_id, release_pos }  => { self.handle_link_unlink_task(task_id, release_pos); },
            AppAction::DisplayToast(toast)                      => { self.handle_display_toast(toast, ctx.clone()) },
            AppAction::RemoveToast(toast_id)                    => { self.handle_remove_toast(toast_id) },
            AppAction::Quit                                     => { ctx.send_viewport_cmd(ViewportCommand::Close); }
            AppAction::Save                                     => { self.handle_save() },
            AppAction::SaveAs                                   => { self.handle_save_as() },
            AppAction::SaveSettings                             => { self.handle_save_settings() },
        }
    }

    fn handle_remove_toast(&mut self, toast_id: ToastId) {
        if let Some((tid, _)) = self.toast {
            if toast_id == tid {
                self.toast = None;
            }
        }
    }

    fn handle_save(&self) {

        // Get current file, if any
        let Some(current_file) = self.settings.current_file.as_deref() else {
            self.channel.send(AppAction::SaveAs);
            return;
        };
        // Serialize to yaml
        let yaml = match serde_yaml::to_string(&self.state) {
            Ok(yaml) => yaml,
            Err(err) => {
                log::error!("{err}");
                let action = AppAction::DisplayToast(Toast::error("Failed to serialize"));
                self.channel.send(action);
                return;
            }
        };
        // Write to file
        if let Err(err) = std::fs::write(&current_file, yaml) {
            log::error!("{err}");
            let message = format!("Failed to write to {}", current_file.display());
            let action = AppAction::DisplayToast(Toast::error(message));
            self.channel.send(action);
            return;
        }
        // Display toast
        let action = AppAction::DisplayToast(Toast::success("Saved"));
        self.channel.send(action);
        // Save settings
        self.channel.send(AppAction::SaveSettings);
    }

    fn handle_save_as(&mut self) {
        let picked_file = FileDialog::new()
            .set_title(SAVE_AS_TITLE)
            .set_file_name(DEFAULT_FILE_NAME)
            .save_file();
        if let Some(mut picked_file) = picked_file {
            Self::cleanup_file_name(&mut picked_file);
            self.settings.current_file = Some(picked_file);
            self.channel.send(AppAction::SaveSettings);
            self.channel.send(AppAction::Save);
        }
    }

    fn handle_save_settings(&self) {
        // Creates dir for settings file if necessary
        match file_utils::create_parent_path_of(&self.paths.settings_file) {
            Ok(_) => {},
            Err(err) => {
                log::error!("{err}");
                let action = AppAction::DisplayToast(Toast::error("Failed to create settings file directory"));
                self.channel.send(action);
                return;
            },
        }
        // Serializes settings to yaml
        let yaml = match serde_yaml::to_string(&self.settings) {
            Ok(yaml) => yaml,
            Err(err) => {
                log::error!("{err}");
                let action = AppAction::DisplayToast(Toast::error("Failed to create settings file directory"));
                self.channel.send(action);
                return;
            },
        };
        // Writes yaml to file
        match std::fs::write(&self.paths.settings_file, yaml) {
            Ok(_) => {}
            Err(err) => {
                log::error!("{err}");
                let action = AppAction::DisplayToast(Toast::error("Failed to save settings file"));
                self.channel.send(action);
            }
        };
    }

    /// If a free arrow touches a task, it either adds a dependency or removes it.
    /// It may do nothing it adding the dependency would introduce a cycle.
    fn handle_link_unlink_task(&mut self, parent_id: TaskId, child_pos: Pos2) {
        let Some((child_id, _)) = self.state.tasks.get_at_pos(child_pos) else { return };
        if parent_id == child_id { return }

        // Attempts to add link (dependency)
        let link_added = match self.state.tasks.add_dependency(parent_id, child_id) {
            Ok(added) => added,
            Err(GraphError::TaskNotFound) => return,
            Err(GraphError::CycleDetected) => {
                let toast = Toast::error("Cycle detected");
                self.channel.send(AppAction::DisplayToast(toast));
                return;
            },
        };

        // If already linked, remove the link
        if !link_added {
            match self.state.tasks.remove_dependency(parent_id, child_id) {
                Ok(_) => {},
                Err(err) => unreachable!("{err}"),
            }
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
        self.state.tasks.insert(task);
    }

    fn handle_display_toast(&mut self, toast: Toast, ctx: Context) {
        let toast_id = self.gen_id();
        self.toast = Some((toast_id, toast));
        let sender = self.channel.sender.clone();
        tokio::spawn(async move {
            tokio::time::sleep(TOAST_DURATION).await;
            ctx.request_repaint(); 
            sender.send(AppAction::RemoveToast(toast_id)).unwrap();
        });
    }

    fn cleanup_file_name(path: &mut PathBuf) {
        if path.extension().is_none() {
            path.set_extension(DEFAULT_FILE_EXTENSION);
        }
    }
}

pub struct Channel {
    sender: Sender<AppAction>,
    receiver: Receiver<AppAction>,
}

impl Channel {
    pub fn send(&self, action: AppAction) {
        self.sender.send(action).unwrap();
    }
}

impl Default for Channel {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver,
        }
    }
}

#[derive(Debug)]
pub enum AppAction {
    Save,
    SaveAs,
    SaveSettings,
    Quit,
    AddTask,
    RemoveTask(TaskId),
    LinkUnlinkTask { task_id: TaskId, release_pos: Pos2 },
    DisplayToast(Toast),
    RemoveToast(ToastId),
}

