mod button_story;
mod calendar_story;
mod dropdown_story;
mod icon_story;
mod image_story;
mod input_story;
mod list_story;
mod modal_story;
mod popup_story;
mod progress_story;
mod resizable_story;
mod scrollable_story;
mod switch_story;
mod table_story;
mod text_story;
mod tooltip_story;
mod webview_story;

use std::sync::Arc;

pub use button_story::ButtonStory;
pub use calendar_story::CalendarStory;
pub use dropdown_story::DropdownStory;
pub use icon_story::IconStory;
pub use image_story::ImageStory;
pub use input_story::InputStory;
pub use list_story::ListStory;
pub use modal_story::ModalStory;
pub use popup_story::PopupStory;
pub use progress_story::ProgressStory;
pub use resizable_story::ResizableStory;
pub use scrollable_story::ScrollableStory;
pub use switch_story::SwitchStory;
pub use table_story::TableStory;
pub use text_story::TextStory;
pub use tooltip_story::TooltipStory;
pub use webview_story::WebViewStory;

use gpui::{
    actions, div, prelude::FluentBuilder as _, px, AnyView, AppContext, Div, EventEmitter,
    FocusableView, InteractiveElement, IntoElement, ParentElement, Pixels, Render, SharedString,
    StatefulInteractiveElement, Styled as _, Task, View, ViewContext, VisualContext, WindowContext,
};

use anyhow::{anyhow, Result};
use ui::{
    divider::Divider,
    dock::{Panel, PanelEvent, PanelId, PanelView, TabPanel},
    h_flex,
    label::Label,
    popup_menu::PopupMenu,
    v_flex, Placement,
};

pub fn init(cx: &mut AppContext) {
    input_story::init(cx);
    dropdown_story::init(cx);
    popup_story::init(cx);
}

actions!(story, [PanelInfo]);

pub fn section(title: impl IntoElement, cx: &WindowContext) -> Div {
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
        .child(div().flex_none().w_full().child(title))
}

pub trait Story {
    fn klass() -> &'static str {
        std::any::type_name::<Self>()
    }

    fn title() -> &'static str;
    fn description() -> &'static str;
    fn new_view(cx: &mut WindowContext) -> AnyView;
}

pub struct StoryContainer {
    panel_id: PanelId,
    focus_handle: gpui::FocusHandle,
    name: SharedString,
    description: SharedString,
    width: Option<gpui::Pixels>,
    height: Option<gpui::Pixels>,
    story: Option<AnyView>,
    closeable: bool,
    story_klass: Option<SharedString>,
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

impl EventEmitter<ContainerEvent> for StoryContainer {}

impl StoryContainer {
    pub fn new(closeable: bool, cx: &mut WindowContext) -> Self {
        let focus_handle = cx.focus_handle();

        Self {
            panel_id: PanelId::new(),
            focus_handle,
            name: SharedString::default(),
            description: SharedString::default(),
            width: None,
            height: None,
            story: None,
            story_klass: None,
            closeable,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_panel<S: Story>(
        tab_panel: View<TabPanel>,
        placement: Option<Placement>,
        size: Option<Pixels>,
        closeable: bool,
        cx: &mut WindowContext,
    ) -> Task<Result<View<Self>>> {
        let name = S::title();
        let description = S::description();
        let story = S::new_view(cx);
        let story_klass = S::klass();

        cx.spawn(|mut cx| async move {
            tab_panel.update(&mut cx, |panel, cx| {
                let view = cx.new_view(|cx| {
                    let mut story = Self::new(closeable, cx).story(story, story_klass);
                    story.name = name.into();
                    story.description = description.into();
                    story
                });
                if let Some(placement) = placement {
                    panel.add_panel_at(Arc::new(view.clone()), placement, size, cx);
                } else {
                    panel.add_panel(Arc::new(view.clone()), cx);
                }
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

    pub fn story(mut self, story: AnyView, story_klass: impl Into<SharedString>) -> Self {
        self.story = Some(story);
        self.story_klass = Some(story_klass.into());
        self
    }
}

impl Panel for StoryContainer {
    fn panel_name() -> &'static str {
        "StoryContainer"
    }

    fn panel_id(&self) -> PanelId {
        self.panel_id
    }

    fn deserialize(
        _: View<ui::dock::DockArea>,
        panel_id: PanelId,
        cx: &mut ViewContext<TabPanel>,
    ) -> Task<Result<Box<dyn PanelView>>> {
        if let Some(data) = LocalStorage::get_panel(panel_id) {
            if let Ok(container) = Self::deserialize_from(&data, cx) {
                let view = cx.new_view(|_| container);
                return Task::Ready(Some(Ok(Box::new(view))));
            }
        }

        Task::Ready(None)
    }

    fn title(&self, _cx: &WindowContext) -> SharedString {
        self.name.clone()
    }

    fn closeable(&self, _cx: &WindowContext) -> bool {
        self.closeable
    }

    fn popup_menu(&self, menu: PopupMenu, _cx: &WindowContext) -> PopupMenu {
        menu.menu("Panel Info", Box::new(PanelInfo))
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct StoryContainerData {
    panel_id: PanelId,
    kind: String,
}

impl StoryContainer {
    fn serialize_to(&self) -> Result<String> {
        let data = StoryContainerData {
            panel_id: self.panel_id,
            kind: self.story_klass.as_ref().unwrap().to_string(),
        };

        Ok(serde_json::to_string(&data)?)
    }

    fn deserialize_from(data: &str, cx: &mut WindowContext) -> Result<Self> {
        let data = serde_json::from_str::<StoryContainerData>(data)?;

        macro_rules! story_for {
            ($p:ident) => {
                (
                    $p::title(),
                    $p::description(),
                    $p::new_view(cx),
                    $p::klass(),
                )
            };
        }

        let (title, description, view, klass) = match data.kind.as_str() {
            "ButtonStory" => story_for!(ButtonStory),
            "CalendarStory" => story_for!(CalendarStory),
            "DropdownStory" => story_for!(DropdownStory),
            "IconStory" => story_for!(IconStory),
            "ImageStory" => story_for!(ImageStory),
            "InputStory" => story_for!(InputStory),
            "ListStory" => story_for!(ListStory),
            "ModalStory" => story_for!(ModalStory),
            "PopupStory" => story_for!(PopupStory),
            "ProgressStory" => story_for!(ProgressStory),
            "ResizableStory" => story_for!(ResizableStory),
            "ScrollableStory" => story_for!(ScrollableStory),
            "SwitchStory" => story_for!(SwitchStory),
            "TableStory" => story_for!(TableStory),
            "TextStory" => story_for!(TextStory),
            "TooltipStory" => story_for!(TooltipStory),
            _ => return Err(anyhow!("Unknown story kind: {}", data.kind)),
        };

        let mut container = Self::new(false, cx).story(view, klass);
        container.name = title.into();
        container.description = description.into();

        Ok(container)
    }
}

impl EventEmitter<PanelEvent> for StoryContainer {}
impl Render for StoryContainer {
    fn render(&mut self, _: &mut ViewContext<Self>) -> impl IntoElement {
        v_flex()
            .id("story-container")
            .size_full()
            .overflow_scroll()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .p_4()
                    .child(Label::new(self.description.clone()).text_size(px(16.0)))
                    .child(Divider::horizontal().label("This is a divider")),
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

pub struct LocalStorage {}

impl LocalStorage {
    fn path(key: &str) -> std::path::PathBuf {
        std::env::home_dir().unwrap().join(".gpui-story").join(key)
    }

    fn get_panel(panel_id: PanelId) -> Option<String> {
        let key = Self::path(&format!("panel-{}", panel_id));
        std::fs::read_to_string(key).ok()
    }

    fn set_panel(panel_id: PanelId, value: String) -> Result<()> {
        let key = Self::path(&format!("panel-{}", panel_id));
        std::fs::write(key, value).map_err(Into::into)
    }
}
