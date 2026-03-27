use crate::{Task, TaskGraph};
use std::ops::Range;
use eframe::{App, CreationContext};
use rand::RngExt;
use egui::pos2;
use egui::{
    vec2, CentralPanel, Color32, Grid, Id, MenuBar, Painter, Panel, Pos2, ScrollArea, Stroke, Ui, ViewportCommand
};


const LINE_STROKE: Stroke = Stroke { width: 2.0, color: Color32::WHITE };
const POS_OFFSET_RANGE: Range<f32>= 0.0..40.0;

pub struct TodoskyApp {
    tasks: TaskGraph,
}

impl App for TodoskyApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.show_top_panel(ui);
        self.show_right_panel(ui);
        self.show_center_panel(ui);
    }
}

impl TodoskyApp {
    pub fn new(_ctx: &CreationContext) -> Self {
        Self {
            tasks: TaskGraph::default(),
        }
    }

    /// Center of the viewport in which new tasks in the graph will be created.
    fn viewport_center(&self) -> Pos2 {
        pos2(512.0, 512.0)
    }

    /// Top panel, which includes the menu bar (File, Edit, Help etc)
    fn show_top_panel(&self, ui: &mut Ui) {
        Panel::top("top_panel")
            .min_size(32.0)
            .show_inside(ui, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    MenuBar::new().ui(ui, |ui| {
                        ui.menu_button("File", |ui| {
                            if ui.button("Quit").clicked() {
                                ui.send_viewport_cmd(ViewportCommand::Close);
                            }
                        });
                    });
                });
            });
    }

    /// Center panel, which includes draggable tasks
    fn show_center_panel(&mut self, ui: &mut Ui) {
        CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Task Graph");
                self.paint_free_arrows(ui.painter());
                self.paint_dependency_arrows(ui.painter());
                for (task_id, task) in self.tasks.iter_mut() {
                    let task_id = Id::new(task_id);
                    task.show_as_node(task_id, ui);
                }
            });
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
            self.handle_add_task();
        }
        // Tasks in vertical list
        Grid::new("vertical_task_list").show(ui, |ui| {
            self.tasks.retain(|_, task| {
                let task_deleted = task.show_as_row(ui);
                ui.end_row();
                !task_deleted
            });
        });
    }

    /// Paints line coming out of task that has no outgoing connection
    fn paint_free_arrows(&self, painter: &Painter) {
        for (_, task) in self.tasks.iter() {
            let task_center = task.rect().center();
            if let Some(arrow_pos) = task.arrow_pos {
                painter.line_segment([task_center, arrow_pos], LINE_STROKE);
            }
        }
    }

    /// Paints lines that connect tasks in the center pane
    fn paint_dependency_arrows(&self, painter: &Painter) {
        for (task_id, task_deps) in self.tasks.dependencies() {
            let task = self.tasks.get(task_id).unwrap();
            let task_pos = task.rect().center();
            for dep_task_id in task_deps.iter().copied() {
                let dep_task = self.tasks.get(dep_task_id).unwrap();
                let dep_task_pos = dep_task.rect().center();
                painter.line_segment([task_pos, dep_task_pos], LINE_STROKE);
            }
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
