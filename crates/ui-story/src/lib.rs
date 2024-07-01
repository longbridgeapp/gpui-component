mod button_story;
mod checkbox_story;
mod dropdown_story;
mod input_story;
mod list_story;
mod picker_story;
mod popover_story;
mod switch_story;
mod tooltip_story;

pub use button_story::ButtonStory;
pub use checkbox_story::CheckboxStory;
pub use dropdown_story::DropdownStory;
pub use input_story::InputStory;
pub use list_story::ListStory;
pub use picker_story::PickerStory;
pub use popover_story::PopoverStory;
pub use switch_story::SwitchStory;
pub use tooltip_story::TooltipStory;

use gpui::{
    div, prelude::FluentBuilder as _, px, AnyElement, AnyView, AppContext, EventEmitter,
    FocusableView, IntoElement, ParentElement, Render, SharedString, Styled as _, Task, View,
    ViewContext, VisualContext, WindowContext,
};
use workspace::{
    dock::{DockPosition, Panel, PanelEvent},
    item::{Item, ItemEvent},
    Workspace, WorkspaceId,
};

use anyhow::Result;
use ui::{divider::Divider, label::Label, v_flex};

pub fn story_case(
    name: &'static str,
    description: &'static str,
    cx: &mut WindowContext,
) -> StoryContainer {
    StoryContainer::new(name, description, cx)
}

pub struct StoryContainer {
    focus_handle: gpui::FocusHandle,
    name: SharedString,
    description: SharedString,
    position: DockPosition,
    width: Option<gpui::Pixels>,
    height: Option<gpui::Pixels>,
    active: bool,
    story: Option<AnyView>,
}

impl FocusableView for StoryContainer {
    fn focus_handle(&self, _: &AppContext) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<PanelEvent> for StoryContainer {}

impl Panel for StoryContainer {
    fn persistent_name() -> &'static str {
        "story-container"
    }

    fn position(&self, _: &WindowContext) -> workspace::dock::DockPosition {
        self.position
    }

    fn can_position(&self, _: workspace::dock::DockPosition, _: &WindowContext) -> bool {
        true
    }

    fn set_position(&mut self, position: workspace::dock::DockPosition, _: &mut WindowContext) {
        self.position = position;
    }

    fn size(&self, _: &WindowContext) -> gpui::Pixels {
        match self.position {
            DockPosition::Left | DockPosition::Right => self.width.unwrap_or(px(360.)),
            DockPosition::Bottom => self.height.unwrap_or(px(360.)),
        }
    }

    fn set_size(&mut self, size: Option<gpui::Pixels>, _: &mut WindowContext) {
        match self.position {
            DockPosition::Left | DockPosition::Right => self.width = size,
            DockPosition::Bottom => self.height = size,
        }
    }

    fn set_active(&mut self, active: bool, _: &mut WindowContext) {
        self.active = active;
    }

    fn starts_open(&self, _cx: &WindowContext) -> bool {
        true
    }
}

#[derive(Debug)]
pub enum ContainerEvent {
    Close,
}

impl Item for StoryContainer {
    type Event = ContainerEvent;

    fn tab_content(
        &self,
        _params: workspace::item::TabContentParams,
        _cx: &WindowContext,
    ) -> AnyElement {
        Label::new(self.name.clone()).into_any_element()
    }

    fn deactivated(&mut self, _: &mut ViewContext<Self>) {
        self.active = false;
    }

    fn workspace_deactivated(&mut self, _cx: &mut ViewContext<Self>) {
        self.active = false;
    }

    fn clone_on_split(
        &self,
        _: Option<WorkspaceId>,
        cx: &mut ViewContext<Self>,
    ) -> Option<View<Self>> {
        Some(cx.new_view(|cx| {
            Self::new(self.name.clone(), self.description.clone(), cx)
                .story(self.story.clone().unwrap())
        }))
    }

    fn to_item_events(event: &Self::Event, mut f: impl FnMut(ItemEvent)) {
        match event {
            ContainerEvent::Close => f(ItemEvent::CloseItem),
        }
    }
}

impl EventEmitter<ContainerEvent> for StoryContainer {}

impl StoryContainer {
    pub fn new(
        name: impl Into<SharedString>,
        description: impl Into<SharedString>,
        cx: &mut WindowContext,
    ) -> Self {
        let focus_handle = cx.focus_handle();

        Self {
            focus_handle,
            name: name.into(),
            description: description.into(),
            width: None,
            height: None,
            position: DockPosition::Left,
            active: false,
            story: None,
        }
    }

    pub fn open(
        name: impl Into<SharedString>,
        description: impl Into<SharedString>,
        story: AnyView,
        workspace: View<Workspace>,
        cx: &mut WindowContext,
    ) -> Task<Result<View<Self>>> {
        let pane = workspace.read(cx).active_pane().clone();
        let name = name.into();
        let description = description.into();

        cx.spawn(|mut cx| async move {
            pane.update(&mut cx, |pane, cx| {
                let view = cx.new_view(|cx| Self::new(name, description, cx).story(story));

                pane.add_item(Box::new(view.clone()), true, true, None, cx);
                view
            })
        })
    }

    pub fn width(mut self, width: gpui::Pixels) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: gpui::Pixels) -> Self {
        self.height = Some(height);
        self
    }

    pub fn position(mut self, position: DockPosition) -> Self {
        self.position = position;
        self
    }

    pub fn story(mut self, story: AnyView) -> Self {
        self.story = Some(story);
        self
    }
}

impl Render for StoryContainer {
    fn render(&mut self, _: &mut ViewContext<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .gap_6()
            .p_2()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(Label::new(self.name.clone()).text_size(px(24.0)))
                    .child(Label::new(self.description.clone()).text_size(px(16.0))),
            )
            .child(Divider::horizontal())
            .when_some(self.story.clone(), |this, story| {
                this.child(v_flex().size_full().child(story))
            })
    }
}
