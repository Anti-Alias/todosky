use egui::{Color32, Frame, Rect, RichText, Ui, UiBuilder};

const MARGIN: i8 = 2;
const CORNER_RADIUS: u8 = 3;
const TEXT_COLOR: Color32 = Color32::WHITE;

pub type ToastId = u32;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Toast {
    pub message: String,
    pub kind: ToastKind,
}

impl Toast {
    pub fn new(message: impl Into<String>, kind: ToastKind) -> Self {
        Self {
            message: message.into(),
            kind,
        }
    }
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: ToastKind::Success,
        }
    }
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: ToastKind::Error,
        }
    }
    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: ToastKind::Warning,
        }
    }

    pub fn show(&self, ui: &mut Ui) {
        let area = Rect {
            min: ui.cursor().min,
            max: ui.cursor().max,
        };
        let builder = UiBuilder::new().max_rect(area);
        ui.scope_builder(builder, |ui| {
            self.show_contents(ui);
        });
    }

    fn show_contents(&self, ui: &mut Ui) {
        Frame::NONE
            .fill(self.kind.color())
            .corner_radius(CORNER_RADIUS)
            .inner_margin(MARGIN)
            .show(ui, |ui| {
                let text = RichText::new(&self.message).color(TEXT_COLOR);
                ui.label(text);
            });
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ToastKind {
    Success,
    Error,
    Warning,
}

impl ToastKind {
    pub fn color(self) -> Color32 {
        match self {
            ToastKind::Success  => Color32::DARK_GREEN,
            ToastKind::Error    => Color32::DARK_RED,
            ToastKind::Warning  => Color32::YELLOW,
        }
    }
}
