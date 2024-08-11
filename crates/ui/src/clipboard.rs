use gpui::{
    prelude::FluentBuilder, AnyElement, ClipboardItem, ElementId, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, WindowContext,
};

use crate::{button::Button, h_flex, IconName, Sizable};

#[derive(IntoElement)]
pub struct Clipboard {
    id: ElementId,
    value: SharedString,
    content_builder: Option<Box<dyn Fn(&mut WindowContext) -> AnyElement>>,
    copied_callback: Option<Box<dyn Fn(SharedString, &mut WindowContext)>>,
}

impl Clipboard {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            value: "".into(),
            content_builder: None,
            copied_callback: None,
        }
    }

    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = value.into();
        self
    }

    pub fn content<E, F>(mut self, element_builder: F) -> Self
    where
        E: IntoElement,
        F: Fn(&mut WindowContext) -> E + 'static,
    {
        self.content_builder = Some(Box::new(move |cx| element_builder(cx).into_any_element()));
        self
    }

    pub fn on_copied<F>(mut self, handler: F) -> Self
    where
        F: Fn(SharedString, &mut WindowContext) + 'static,
    {
        self.copied_callback = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Clipboard {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        h_flex()
            .gap_1()
            .items_center()
            .when_some(self.content_builder, |this, builder| {
                this.child(builder(cx))
            })
            .child(
                Button::new(self.id, cx)
                    .icon(IconName::Copy)
                    .ghost()
                    .xsmall()
                    .on_click(move |_, cx| {
                        cx.stop_propagation();
                        cx.write_to_clipboard(ClipboardItem::new(self.value.to_string()));

                        if let Some(copied) = &self.copied_callback {
                            copied(self.value.clone(), cx);
                        }
                    }),
            )
    }
}
