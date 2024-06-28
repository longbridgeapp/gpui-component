mod button_story;
mod checkbox_story;
mod dropdown_story;
mod input_story;
mod picker_story;
mod switch_story;

use gpui::{
    div, prelude::FluentBuilder as _, px, AnyElement, IntoElement, ParentElement, Render,
    RenderOnce, SharedString, StatefulInteractiveElement as _, Styled as _, View, ViewContext,
    VisualContext, WindowContext,
};

use std::fmt::{self, Display, Formatter};
use ui::{
    divider::Divider,
    label::Label,
    tab::{Tab, TabBar},
    Selectable,
};

use button_story::ButtonStory;
use checkbox_story::CheckboxStory;
use dropdown_story::DropdownStory;
use input_story::InputStory;
use picker_story::PickerStory;
use switch_story::SwitchStory;

pub fn story_case(name: &'static str, description: &'static str) -> StoryContainer {
    StoryContainer::new(name, description)
}

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
    fn render(self, _: &mut WindowContext) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_6()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(Label::new(self.name).text_size(px(24.0)))
                    .child(Label::new(self.description).text_size(px(16.0))),
            )
            .child(Divider::horizontal())
            .children(self.children)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum StoryType {
    Button,
    Input,
    Checkbox,
    Switch,
    Picker,
    Dropdown,
}

impl Display for StoryType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Button => write!(f, "Button"),
            Self::Input => write!(f, "Input"),
            Self::Checkbox => write!(f, "Checkbox"),
            Self::Switch => write!(f, "Switch"),
            Self::Picker => write!(f, "Picker"),
            Self::Dropdown => write!(f, "Dropdown"),
        }
    }
}

pub struct Stories {
    active: StoryType,

    button_story: View<ButtonStory>,
    input_story: View<InputStory>,
    checkbox_story: View<CheckboxStory>,
    switch_story: View<SwitchStory>,
    picker_story: View<PickerStory>,
    dropdown_story: View<DropdownStory>,
}

impl Stories {
    fn new(cx: &mut ViewContext<Self>) -> Self {
        Self {
            active: StoryType::Button,
            button_story: cx.new_view(|_| ButtonStory {}),
            checkbox_story: cx.new_view(|cx| CheckboxStory::new(cx)),
            input_story: cx.new_view(|cx| InputStory::new(cx)),
            switch_story: cx.new_view(|cx| SwitchStory::new(cx)),
            picker_story: cx.new_view(|cx| PickerStory::new(cx)),
            dropdown_story: cx.new_view(|cx| DropdownStory::new(cx)),
        }
    }

    pub fn view(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(Self::new)
    }

    fn set_active(&mut self, ty: StoryType, cx: &mut ViewContext<Self>) {
        self.active = ty;
        cx.notify();
    }

    fn tabs(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap_4()
            .w_full()
            .child(TabBar::new("story-tabs").children(vec![
                self.tab("story-button", StoryType::Button, cx),
                self.tab("story-input", StoryType::Input, cx),
                self.tab("story-checkbox", StoryType::Checkbox, cx),
                self.tab("story-switch", StoryType::Switch, cx),
                self.tab("story-picker", StoryType::Picker, cx),
                self.tab("story-dropdown", StoryType::Dropdown, cx),
            ]))
    }

    fn tab(&self, id: &str, ty: StoryType, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let name = format!("{}", ty);
        let is_active = ty == self.active;

        Tab::new(SharedString::from(id.to_string()), name)
            .selected(is_active)
            .on_click(cx.listener(move |this, _, cx| {
                this.set_active(ty, cx);
            }))
    }
}

impl Render for Stories {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_4()
            .child(self.tabs(cx))
            .map(|this| match self.active {
                StoryType::Button => this.child(self.button_story.clone()),
                StoryType::Input => this.child(self.input_story.clone()),
                StoryType::Checkbox => this.child(self.checkbox_story.clone()),
                StoryType::Switch => this.child(self.switch_story.clone()),
                StoryType::Picker => this.child(self.picker_story.clone()),
                StoryType::Dropdown => this.child(self.dropdown_story.clone()),
            })
    }
}
