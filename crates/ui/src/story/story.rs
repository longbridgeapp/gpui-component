use core::fmt;
use std::fmt::{Display, Formatter};

use gpui::{
    div, prelude::FluentBuilder as _, px, AnyElement, ElementId, InputHandler, IntoElement,
    ParentElement, Render, RenderOnce, SharedString, Styled as _, ViewContext, VisualContext,
    WindowContext,
};

use crate::{button::Button, disableable::Clickable as _, label::Label};

use super::{button_story::ButtonStory, input_story::InputStory};

#[derive(IntoElement)]
pub struct StoryContainer {
    name: SharedString,
    description: SharedString,
    children: Vec<AnyElement>,
}

impl ParentElement for StoryContainer {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements)
    }
}

impl StoryContainer {
    pub fn new(name: impl Into<SharedString>, description: impl Into<SharedString>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            children: Vec::new(),
        }
    }
}

impl RenderOnce for StoryContainer {
    fn render(self, _cx: &mut WindowContext) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(Label::new(self.name).text_size(px(24.0)))
                    .child(Label::new(self.description).text_size(px(16.0))),
            )
            .children(self.children)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum StoryType {
    Button,
    Input,
}

impl Display for StoryType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Button => write!(f, "Button"),
            Self::Input => write!(f, "Input"),
        }
    }
}

pub struct Stories {
    active: StoryType,
}

impl Stories {
    pub fn new() -> Self {
        Self {
            active: StoryType::Input,
        }
    }

    fn set_active(&mut self, ty: StoryType, cx: &mut ViewContext<Self>) {
        self.active = ty;
        dbg!("--------------------- set_active: {}", ty);
        cx.notify();
    }

    fn render_story_buttons(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap_4()
            .child(self.swith_button(StoryType::Button, cx))
            .child(self.swith_button(StoryType::Input, cx))
    }

    fn swith_button(&self, ty: StoryType, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let name = format!("{}", ty);
        Button::new(ElementId::Name(SharedString::from(name.clone())), name)
            .on_click(cx.listener(move |this, _, cx| {
                dbg!("--------------------- on_click: {}", ty);
                this.set_active(ty, cx);
            }))
            .style(crate::button::ButtonStyle::Secondary)
    }
}

impl Default for Stories {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for Stories {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let nav_buttons = self.render_story_buttons(cx);

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(nav_buttons)
            .map(|this| match self.active {
                StoryType::Button => this.child(cx.new_view(|cx| ButtonStory {})),
                StoryType::Input => this.child(cx.new_view(|cx| InputStory {})),
            })
    }
}
