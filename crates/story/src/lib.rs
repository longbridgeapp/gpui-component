mod button_story;
mod checkbox_story;
mod dropdown_story;
mod image_story;
mod input_story;
mod list_story;
mod picker_story;
mod popover_story;
mod switch_story;
mod table_story;
mod tooltip_story;

pub use button_story::ButtonStory;
pub use checkbox_story::CheckboxStory;
pub use dropdown_story::DropdownStory;
pub use image_story::ImageStory;
pub use input_story::InputStory;
pub use list_story::ListStory;
pub use picker_story::PickerStory;
pub use popover_story::PopoverStory;
pub use switch_story::SwitchStory;
pub use table_story::TableStory;
pub use tooltip_story::TooltipStory;

use gpui::{
    div, prelude::FluentBuilder as _, px, AnyElement, AnyView, AppContext, Div, EventEmitter,
    FocusableView, InteractiveElement, IntoElement, ParentElement, Pixels, Render,
    SharedString, StatefulInteractiveElement, Styled as _, Task, View, ViewContext, VisualContext,
    WindowContext,
};
use workspace::{
    dock::{DockPosition, Panel, PanelEvent},
    item::{Item, ItemEvent},
    Workspace, WorkspaceId,
};

use anyhow::Result;
use ui::{divider::Divider, h_flex, label::Label, v_flex};

pub fn init(cx: &mut AppContext) {
    input_story::init(cx);
}

pub fn story_case(
    name: &'static str,
    description: &'static str,
    cx: &mut WindowContext,
) -> StoryContainer {
    StoryContainer::new(name, description, cx)
}

pub fn section(title: impl Into<SharedString>, cx: &WindowContext) -> Div {
    use ui::theme::ActiveTheme;
    let theme = cx.theme();

    h_flex()
        .items_center()
        .gap_4()
        .p_4()
        .w_full()
        .rounded_lg()
        .border_1()
        .border_color(theme.border)
        .flex_wrap()
        .justify_around()
        .child(div().flex_none().w_full().child(title.into()))
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

    pub fn add_pane(
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

    pub fn add_panel(
        story: AnyView,
        workspace: View<Workspace>,
        position: DockPosition,
        size: gpui::Pixels,
        cx: &mut WindowContext,
    ) {
        workspace.update(cx, |workspace, cx| {
            let panel = cx.new_view(|cx| MyPanel::new(story, position, size, cx));
            workspace.add_panel(panel, cx)
        });
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
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .p_4()
                    .child(Label::new(self.name.clone()).text_size(px(24.0)))
                    .child(Label::new(self.description.clone()).text_size(px(16.0)))
                    .child(Divider::horizontal()),
            )
            .when_some(self.story.clone(), |this, story| {
                this.child(
                    v_flex()
                        .id("story-children")
                        .overflow_scroll()
                        .size_full()
                        .p_4()
                        .child(story),
                )
            })
    }
}

struct MyPanel {
    focus_handle: gpui::FocusHandle,
    view: AnyView,
    _position: DockPosition,
    width: Option<Pixels>,
    height: Option<Pixels>,
}

impl MyPanel {
    fn new(view: AnyView, position: DockPosition, size: Pixels, cx: &mut WindowContext) -> Self {
        let mut this = Self {
            focus_handle: cx.focus_handle(),
            view,
            _position: position,
            width: None,
            height: None,
        };
        this.update_size(size);
        this
    }

    fn update_size(&mut self, size: Pixels) {
        match self._position {
            DockPosition::Bottom => self.height = Some(size),
            DockPosition::Left | DockPosition::Right => self.width = Some(size),
        }
    }
}

impl Panel for MyPanel {
    fn persistent_name() -> &'static str {
        "my-panel"
    }

    fn can_position(&self, position: DockPosition) -> bool {
        match self._position {
            DockPosition::Bottom => matches!(position, DockPosition::Bottom),
            DockPosition::Left | DockPosition::Right => {
                matches!(position, DockPosition::Left | DockPosition::Right)
            }
        }
    }

    fn position(&self, _cx: &WindowContext) -> DockPosition {
        self._position
    }

    fn set_position(&mut self, position: DockPosition, cx: &mut ViewContext<Self>) {
        self._position = position;
        cx.notify()
    }

    fn size(&self, _cx: &WindowContext) -> gpui::Pixels {
        match self._position {
            DockPosition::Bottom => self.height.unwrap_or(px(100.)),
            DockPosition::Left | DockPosition::Right => self.width.unwrap_or(px(100.)),
        }
    }

    fn set_size(&mut self, size: Option<gpui::Pixels>, cx: &mut ViewContext<Self>) {
        if let Some(size) = size {
            self.update_size(size)
        }
        cx.notify();
    }
}

impl EventEmitter<PanelEvent> for MyPanel {}
impl FocusableView for MyPanel {
    fn focus_handle(&self, _: &AppContext) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}
impl Render for MyPanel {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .id("my-panel")
            .track_focus(&self.focus_handle)
            .size_full()
            .child(self.view.clone())
    }
}
