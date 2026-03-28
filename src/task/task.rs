use egui::{Color32, Frame, Id, InnerResponse, Label, Layout, PointerButton, Pos2, Rect, Response, Sense, Stroke, Ui, UiBuilder, Vec2, Widget};
use slotmap::{new_key_type};

new_key_type! { pub struct TaskId; }

const TASK_MARGIN: i8 = 5;
const TASK_CORNER_RADIUS: u8 = 3;
const TASK_SIZE: Vec2 = Vec2::new(150.0, 30.0);
const TASK_STROKE: Stroke = Stroke { width: 1.0, color: Color32::WHITE };

#[derive(Clone, PartialEq, Debug)]
pub struct Task {
    pub name: String,
    pub description: Option<String>,
    pub pos: Pos2,
    /// When dragging with RMB, determines end point of line to be drawn
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
    pub fn show_as_node(&mut self, id: Id, ui: &mut Ui) -> InnerResponse<TaskResponse> {

        // Renders task in a draggable area
        let rect = self.rect();
        let builder = UiBuilder::default()
            .id(id)
            .sense(Sense::DRAG)
            .max_rect(rect);
        let response = ui.scope_builder(builder, |ui| {
            Frame::NONE
                .stroke(TASK_STROKE)
                .inner_margin(TASK_MARGIN)
                .fill(ui.visuals().window_fill)
                .corner_radius(TASK_CORNER_RADIUS)
                .show(ui, |ui| {
                    self.show_content(ui);
                });
        }).response;

        // Handles response
        let inner_resp = self.handle_dragging(&response, ui);
        if response.hovered() {
            ui.set_cursor_icon(egui::CursorIcon::Grabbing);
        }
        InnerResponse::new(inner_resp, response)
    }

    fn handle_dragging(&mut self, response: &Response, ui: &mut Ui) -> TaskResponse {

        let Some(pointer_pos) = ui.pointer_latest_pos() else {
            return TaskResponse::None
        };

        // LMB dragging logic
        if response.dragged_by(PointerButton::Primary) {
            self.pos += response.drag_delta();
        }

        // RMB dragging logic. Drags free arrow.
        if response.dragged_by(PointerButton::Secondary) {
            let global_to_local = ui
                .layer_transform_to_global(ui.layer_id())
                .unwrap()
                .inverse();
            let pointer_pos = global_to_local * pointer_pos;
            self.arrow_pos = Some(pointer_pos);
        }

        // RMB drag release logic. Removes free arrow.
        if response.drag_stopped_by(PointerButton::Secondary) {
            if let Some(arrow_pos) = self.arrow_pos.take() {
                return TaskResponse::ArrowReleased { release_pos: arrow_pos };
            }
        }
        TaskResponse::None
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

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub enum TaskResponse {
    #[default]
    None,
    ArrowReleased { release_pos: Pos2 },
}
