use eframe::{App, CreationContext, Frame};
use egui::{CentralPanel, Color32, Context, Layout, MenuBar, Painter, SidePanel, Stroke, TopBottomPanel, ViewportCommand};

use crate::{Task, TaskGraph};

pub struct TodoskyApp {
    graph: TaskGraph,
}

impl App for TodoskyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        self.top_panel(ctx);
        self.center_panel(ctx);
        self.right_panel(ctx);
    }
}

impl TodoskyApp {

    pub fn new(_ctx: &CreationContext) -> Self {
        let mut graph = TaskGraph::default();
        let first_id = graph.insert(Task::new("First Task"));
        let second_id = graph.insert(Task::new("Second Task"));
        let third_id = graph.insert(Task::new("Third Task"));
        graph.add_dependency(second_id, first_id);
        graph.add_dependency(third_id, first_id);
        Self {
            graph,
        }
    }

    /// Top panel, which includes the menu bar (File, Edit, Help etc)
    fn top_panel(&self, ctx: &Context) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(ViewportCommand::Close);
                    }
                });
            });
        });
    }

    /// Center panel, which includes draggable tasks
    fn center_panel(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            self.paint_connection_lines(ui.painter());
            for (_, task) in self.graph.iter_mut() {
                task.show(ui);
            }
        });
    }

    fn paint_connection_lines(&self, painter: &Painter) {
        let stroke = Stroke { width: 2.0, color: Color32::WHITE };
        for (task_id, task_deps) in self.graph.dependencies() {
            let task = self.graph.get(task_id).unwrap();
            let task_pos = task.rect().center();
            for dep_task_id in task_deps.iter().copied() {
                let dep_task = self.graph.get(dep_task_id).unwrap(); 
                let dep_task_pos = dep_task.rect().center();
                painter.line_segment([task_pos, dep_task_pos], stroke);
            }
        }
    }

    /// Right panel, which shows details about the currently selected task, if any
    fn right_panel(&self, ctx: &Context) {
        SidePanel::right("right_panel").show(ctx, |ui| {
            ui.with_layout(Layout::top_down_justified(egui::Align::Min), |ui| {
                ui.label("Text!!!");
                ui.label("Text!!!");
                ui.label("Text!!!");
                ui.label("Text!!!");
                ui.label("Text!!!");
                ui.label("Text!!!");
                ui.label("Text!!!");
                ui.label("Text!!!");
            });
        });
    }
}

