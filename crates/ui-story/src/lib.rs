mod button_story;
mod checkbox_story;
mod dropdown_story;
// mod input_story;
// mod list_story;
// mod picker_story;
// mod popover_story;
// mod switch_story;
// mod tooltip_story;

use gpui::{
    div, prelude::FluentBuilder as _, px, AnyElement, AnyView, AppContext, Element, EventEmitter,
    FocusableView, IntoElement, ParentElement, Render, RenderOnce, SharedString,
    StatefulInteractiveElement as _, StyleRefinement, Styled as _, View, ViewContext,
    VisualContext, WindowContext,
};
use workspace::dock::{DockPosition, Panel, PanelEvent};

use std::fmt::{self, Display, Formatter};
use ui::{
    divider::Divider,
    label::Label,
    tab::{Tab, TabBar},
    Icon, IconName, Selectable, StyledExt,
};

pub use button_story::ButtonStory;
pub use checkbox_story::CheckboxStory;
pub use dropdown_story::DropdownStory;
// use input_story::InputStory;
// use list_story::ListStory;
// use picker_story::PickerStory;
// use popover_story::PopoverStory;
// use switch_story::SwitchStory;
// use tooltip_story::TooltipStory;

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
    children: Vec<AnyElement>,
    position: DockPosition,
    width: Option<gpui::Pixels>,
    height: Option<gpui::Pixels>,
    active: bool,
    story: Option<AnyView>,
}

impl FocusableView for StoryContainer {
    fn focus_handle(&self, cx: &AppContext) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<PanelEvent> for StoryContainer {}

impl Panel for StoryContainer {
    fn persistent_name() -> &'static str {
        "story-container"
    }

    fn position(&self, cx: &WindowContext) -> workspace::dock::DockPosition {
        self.position
    }

    fn can_position(&self, position: workspace::dock::DockPosition, cx: &WindowContext) -> bool {
        true
    }

    fn set_position(&mut self, position: workspace::dock::DockPosition, cx: &mut WindowContext) {
        self.position = position;
    }

    fn size(&self, cx: &WindowContext) -> gpui::Pixels {
        match self.position {
            DockPosition::Left | DockPosition::Right => self.width.unwrap_or(px(360.)),
            DockPosition::Bottom => self.height.unwrap_or(px(360.)),
        }
    }

    fn set_size(&mut self, size: Option<gpui::Pixels>, cx: &mut WindowContext) {
        match self.position {
            DockPosition::Left | DockPosition::Right => self.width = size,
            DockPosition::Bottom => self.height = size,
        }
    }

    fn set_active(&mut self, active: bool, cx: &mut WindowContext) {
        self.active = active;
    }

    fn starts_open(&self, _cx: &WindowContext) -> bool {
        true
    }
}

impl ParentElement for StoryContainer {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements)
    }
}

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
            children: Vec::new(),
            width: None,
            height: None,
            position: DockPosition::Left,
            active: false,
            story: None,
        }
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
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
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
                this.child(story.cached(StyleRefinement::default().v_flex().size_full()))
            })
    }
}

// #[derive(Clone, Copy, PartialEq, Eq, Debug)]
// enum StoryType {
//     Button,
//     Input,
//     Checkbox,
//     Switch,
//     Picker,
//     List,
//     Dropdown,
//     Tooltip,
//     Popover,
// }

// impl Display for StoryType {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         match self {
//             Self::Button => write!(f, "Button"),
//             Self::Input => write!(f, "Input"),
//             Self::Checkbox => write!(f, "Checkbox"),
//             Self::Switch => write!(f, "Switch"),
//             Self::Picker => write!(f, "Picker"),
//             Self::List => write!(f, "List"),
//             Self::Dropdown => write!(f, "Dropdown"),
//             Self::Tooltip => write!(f, "Tooltip"),
//             Self::Popover => write!(f, "Popover"),
//         }
//     }
// }

// pub struct Stories {
//     active: StoryType,

//     button_story: View<ButtonStory>,
//     input_story: View<InputStory>,
//     checkbox_story: View<CheckboxStory>,
//     switch_story: View<SwitchStory>,
//     picker_story: View<PickerStory>,
//     list_story: View<ListStory>,
//     dropdown_story: View<DropdownStory>,
//     tooltip_story: View<TooltipStory>,
//     popover_story: View<PopoverStory>,
// }

// impl Stories {
//     fn new(cx: &mut ViewContext<Self>) -> Self {
//         Self {
//             active: StoryType::Tooltip,
//             button_story: cx.new_view(|_| ButtonStory {}),
//             checkbox_story: cx.new_view(|cx| CheckboxStory::new(cx)),
//             input_story: cx.new_view(|cx| InputStory::new(cx)),
//             switch_story: cx.new_view(|cx| SwitchStory::new(cx)),
//             tooltip_story: cx.new_view(|_| TooltipStory),
//             picker_story: cx.new_view(PickerStory::new),
//             list_story: cx.new_view(|cx| ListStory::new(cx)),
//             dropdown_story: cx.new_view(DropdownStory::new),
//             popover_story: cx.new_view(PopoverStory::new),
//         }
//     }

//     pub fn view(cx: &mut WindowContext) -> View<Self> {
//         cx.new_view(Self::new)
//     }

//     fn set_active(&mut self, ty: StoryType, cx: &mut ViewContext<Self>) {
//         self.active = ty;
//         cx.notify();
//     }

//     fn tabs(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
//         div()
//             .flex()
//             .items_center()
//             .gap_4()
//             .w_full()
//             .child(TabBar::new("story-tabs").children(vec![
//                 self.tab("story-button", StoryType::Button, Some(IconName::Close), cx),
//                 self.tab("story-input", StoryType::Input, None, cx),
//                 self.tab(
//                     "story-checkbox",
//                     StoryType::Checkbox,
//                     Some(IconName::Check),
//                     cx,
//                 ),
//                 self.tab("story-switch", StoryType::Switch, None, cx),
//                 self.tab("story-picker", StoryType::Picker, None, cx),
//                 self.tab("story-list", StoryType::List, None, cx),
//                 self.tab("story-dropdown", StoryType::Dropdown, None, cx),
//                 self.tab("story-tooltip", StoryType::Tooltip, None, cx),
//                 self.tab("story-popover", StoryType::Popover, None, cx),
//             ]))
//     }

//     fn tab(
//         &self,
//         id: &str,
//         ty: StoryType,
//         icon: Option<impl Into<Icon>>,
//         cx: &mut ViewContext<Self>,
//     ) -> impl IntoElement {
//         let name = format!("{}", ty);
//         let is_active = ty == self.active;

//         let tab = Tab::new(SharedString::from(id.to_string()), name.into_any_element())
//             .selected(is_active)
//             .on_click(cx.listener(move |this, _, cx| {
//                 this.set_active(ty, cx);
//             }));

//         if let Some(icon) = icon {
//             tab.prefix(icon.into())
//         } else {
//             tab
//         }
//     }
// }

// impl Render for Stories {
//     fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
//         div()
//             .w_full()
//             .flex()
//             .flex_col()
//             .gap_4()
//             .child(self.tabs(cx))
//             .map(|this| match self.active {
//                 StoryType::Button => this.child(self.button_story.clone()),
//                 StoryType::Input => this.child(self.input_story.clone()),
//                 StoryType::Checkbox => this.child(self.checkbox_story.clone()),
//                 StoryType::Switch => this.child(self.switch_story.clone()),
//                 StoryType::Picker => this.child(self.picker_story.clone()),
//                 StoryType::List => this.child(self.list_story.clone()),
//                 StoryType::Dropdown => this.child(self.dropdown_story.clone()),
//                 StoryType::Tooltip => this.child(self.tooltip_story.clone()),
//                 StoryType::Popover => this.child(self.popover_story.clone()),
//             })
//     }
// }
