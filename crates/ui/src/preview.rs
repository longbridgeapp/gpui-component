use gpui::{div, px, IntoElement, ParentElement as _, RenderOnce, Styled as _};

use crate::{button::ButtonPreview, label::Label};

pub trait Preview {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn new() -> Self;
}

enum PreviewStory {
    Button,
}

#[derive(IntoElement)]
pub struct PreviewBox {
    active_preview: PreviewStory,
}

impl PreviewBox {
    pub fn new() -> Self {
        Self {
            active_preview: PreviewStory::Button,
        }
    }

    pub fn active_preview(&self) -> impl Preview + IntoElement {
        match self.active_preview {
            PreviewStory::Button => ButtonPreview::new(),
        }
    }
}

impl RenderOnce for PreviewBox {
    fn render(self, cx: &mut gpui::WindowContext) -> impl IntoElement {
        let component_preview = self.active_preview();

        let heading = Label::new(component_preview.name()).text_size(px(24.0));
        let description = Label::new(component_preview.description()).multiple_lines();

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(heading)
                    .child(description),
            )
            .child(component_preview)
    }
}
