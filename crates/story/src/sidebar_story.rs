use gpui::{
    px, ParentElement, Render, Styled, View, ViewContext, VisualContext as _, WindowContext,
};

use ui::{
    divider::Divider,
    h_flex,
    prelude::FluentBuilder,
    sidebar::{Sidebar, SidebarGroup, SidebarMenu, SidebarToggleButton},
    theme::ActiveTheme,
    v_flex, Collapsible, ContextModal, IconName,
};

pub struct SidebarStory {
    active_item: Item,
    collapsed: bool,
    focus_handle: gpui::FocusHandle,
}

impl SidebarStory {
    pub fn view(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(Self::new)
    }

    fn new(cx: &mut ViewContext<Self>) -> Self {
        Self {
            active_item: Item::Playground,
            collapsed: false,
            focus_handle: cx.focus_handle(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Item {
    Playground,
    Models,
    Documentation,
    Settings,
    DesignEngineering,
    SalesAndMarketing,
    Travel,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SubItem {
    History,
    Starred,
    Settings,
    Genesis,
    Explorer,
    Quantum,
    Introduction,
    GetStarted,
    Tutorial,
    Changelog,
}

impl Item {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Playground => "Playground",
            Self::Models => "Models",
            Self::Documentation => "Documentation",
            Self::Settings => "Settings",
            Self::DesignEngineering => "Design Engineering",
            Self::SalesAndMarketing => "Sales and Marketing",
            Self::Travel => "Travel",
        }
    }

    pub fn icon(&self) -> IconName {
        match self {
            Self::Playground => IconName::SquareTerminal,
            Self::Models => IconName::Bot,
            Self::Documentation => IconName::BookOpen,
            Self::Settings => IconName::Settings2,
            Self::DesignEngineering => IconName::Frame,
            Self::SalesAndMarketing => IconName::ChartPie,
            Self::Travel => IconName::Map,
        }
    }

    pub fn handler(&self) -> impl Fn(&mut WindowContext) + 'static {
        let item = *self;
        move |cx| {
            cx.push_notification(format!("Clicked on {}", item.label()));
        }
    }

    pub fn items(&self) -> Vec<SubItem> {
        match self {
            Self::Playground => vec![SubItem::History, SubItem::Starred, SubItem::Settings],
            Self::Models => vec![SubItem::Genesis, SubItem::Explorer, SubItem::Quantum],
            Self::Documentation => vec![
                SubItem::Introduction,
                SubItem::GetStarted,
                SubItem::Tutorial,
                SubItem::Changelog,
            ],
            _ => Vec::new(),
        }
    }
}

impl SubItem {
    pub fn label(&self) -> &'static str {
        match self {
            Self::History => "History",
            Self::Starred => "Starred",
            Self::Settings => "Settings",
            Self::Genesis => "Genesis",
            Self::Explorer => "Explorer",
            Self::Quantum => "Quantum",
            Self::Introduction => "Introduction",
            Self::GetStarted => "Get Started",
            Self::Tutorial => "Tutorial",
            Self::Changelog => "Changelog",
        }
    }

    pub fn handler(&self) -> impl Fn(&mut WindowContext) + 'static {
        let item = *self;
        move |cx| {
            cx.push_notification(format!("Clicked on {}", item.label()));
        }
    }
}

impl super::Story for SidebarStory {
    fn title() -> &'static str {
        "Sidebar"
    }

    fn new_view(cx: &mut WindowContext) -> View<impl gpui::FocusableView> {
        Self::view(cx)
    }

    fn zoomable() -> bool {
        true
    }
}
impl gpui::FocusableView for SidebarStory {
    fn focus_handle(&self, _: &gpui::AppContext) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}
impl Render for SidebarStory {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
        let groups: [Vec<Item>; 2] = [
            vec![
                Item::Playground,
                Item::Models,
                Item::Documentation,
                Item::Settings,
            ],
            vec![
                Item::DesignEngineering,
                Item::SalesAndMarketing,
                Item::Travel,
            ],
        ];

        h_flex()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().border)
            .h_full()
            .child(
                Sidebar::left(cx.view())
                    .width(px(280.))
                    .collapsed(self.collapsed)
                    .when(!self.collapsed, |this| {
                        this.header("This is header").footer("This is footer")
                    })
                    .child(SidebarGroup::new("Platform").child(SidebarMenu::new().map(
                        |mut menu| {
                            for item in groups[0].iter() {
                                menu = menu.submenu_with_icon(
                                    item.label(),
                                    item.icon(),
                                    |mut submenu| {
                                        for subitem in item.items() {
                                            submenu = submenu.menu(
                                                subitem.label(),
                                                false,
                                                subitem.handler(),
                                            );
                                        }
                                        submenu
                                    },
                                );
                            }
                            menu
                        },
                    )))
                    .child(SidebarGroup::new("Projects").child(SidebarMenu::new().map(
                        |mut menu| {
                            for item in groups[0].iter() {
                                menu = menu.menu_with_icon(
                                    item.label(),
                                    item.icon(),
                                    self.active_item == *item,
                                    item.handler(),
                                );
                            }
                            menu
                        },
                    ))),
            )
            .child(
                v_flex()
                    .size_full()
                    .gap_4()
                    .p_4()
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .child(
                                SidebarToggleButton::left()
                                    .collapsed(self.collapsed)
                                    .on_click(cx.listener(|this, _, cx| {
                                        this.collapsed = !this.collapsed;
                                        cx.notify();
                                    })),
                            )
                            .child(Divider::vertical().h_4())
                            .child("Building Your Application"),
                    )
                    .child("This content"),
            )
    }
}
