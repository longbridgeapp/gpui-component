use crate::{h_flex, theme::ActiveTheme as _, v_flex, Icon, IconName, StyledExt};
use gpui::{
    div, percentage, prelude::FluentBuilder as _, InteractiveElement as _, IntoElement,
    ParentElement as _, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled as _,
    WindowContext,
};
use std::rc::Rc;

pub struct SidebarMenu {
    items: Vec<SidebarMenuItem>,
}

impl SidebarMenu {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn item(
        &mut self,
        icon: Option<Icon>,
        label: impl Into<SharedString>,
        handler: impl Fn(&mut WindowContext) + 'static,
    ) {
        self.items.push(SidebarMenuItem::Item {
            icon,
            label: label.into(),
            handler: Rc::new(handler),
            active: false,
        });
    }

    pub fn submenu(
        &mut self,
        icon: Option<Icon>,
        label: impl Into<SharedString>,
        items: impl FnOnce(&mut SidebarMenu),
    ) {
        let mut menu = SidebarMenu::new();
        items(&mut menu);
        self.items.push(SidebarMenuItem::Submenu {
            icon,
            label: label.into(),
            items: menu.items,
        });
    }
}

/// A sidebar menu item
#[derive(IntoElement)]
enum SidebarMenuItem {
    Item {
        icon: Option<Icon>,
        label: SharedString,
        handler: Rc<dyn Fn(&mut WindowContext)>,
        active: bool,
    },
    Submenu {
        icon: Option<Icon>,
        label: SharedString,
        items: Vec<SidebarMenuItem>,
    },
}

impl SidebarMenuItem {
    fn is_submenu(&self) -> bool {
        matches!(self, SidebarMenuItem::Submenu { .. })
    }

    fn icon(&self) -> Option<Icon> {
        match self {
            SidebarMenuItem::Item { icon, .. } => icon.clone(),
            SidebarMenuItem::Submenu { icon, .. } => icon.clone(),
        }
    }

    fn label(&self) -> SharedString {
        match self {
            SidebarMenuItem::Item { label, .. } => label.clone(),
            SidebarMenuItem::Submenu { label, .. } => label.clone(),
        }
    }

    fn is_active(&self) -> bool {
        match self {
            SidebarMenuItem::Item { active, .. } => *active,
            SidebarMenuItem::Submenu { .. } => false,
        }
    }

    fn is_open(&self) -> bool {
        match self {
            SidebarMenuItem::Item { .. } => false,
            SidebarMenuItem::Submenu { items, .. } => items.iter().any(|item| item.is_active()),
        }
    }

    fn render_menu_item(
        &self,
        is_submenu: bool,
        is_active: bool,
        is_open: bool,
        cx: &WindowContext,
    ) -> impl IntoElement {
        let handler = match &self {
            SidebarMenuItem::Item { handler, .. } => Some(handler.clone()),
            SidebarMenuItem::Submenu { .. } => None,
        };

        h_flex()
            .id("sidebar-menu-item")
            .flex_shrink_0()
            .h_8()
            .p_2()
            .gap_2()
            .overflow_hidden()
            .items_center()
            .rounded_md()
            .text_sm()
            .cursor_pointer()
            .hover(|this| {
                this.bg(cx.theme().sidebar_accent)
                    .text_color(cx.theme().sidebar_accent_foreground)
            })
            .when(is_active, |this| {
                this.font_medium()
                    .bg(cx.theme().sidebar_accent)
                    .text_color(cx.theme().sidebar_accent_foreground)
            })
            .when(is_open, |this| {
                this.bg(cx.theme().sidebar_accent)
                    .text_color(cx.theme().sidebar_accent_foreground)
            })
            .when_some(self.icon(), |this, icon| this.child(icon.size_4()))
            .child(self.label())
            .when(is_submenu, |this| {
                this.ml_auto().child(
                    Icon::new(IconName::ChevronRight)
                        .size_4()
                        .when(is_open, |this| this.rotate(percentage(90. / 360.))),
                )
            })
            .when_some(handler, |this, handler| {
                this.on_click(move |_, cx| handler(cx))
            })
    }
}

impl RenderOnce for SidebarMenuItem {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let is_submenu = self.is_submenu();
        let is_active = self.is_active();
        let is_open = self.is_open();

        div()
            .child(self.render_menu_item(is_submenu, is_active, is_open, cx))
            .when(is_open, |this| {
                this.map(|this| match self {
                    SidebarMenuItem::Submenu { items, .. } => this.child(
                        v_flex()
                            .border_l_1()
                            .border_color(cx.theme().sidebar_border)
                            .gap_1()
                            .mx_3p5()
                            .px_2p5()
                            .py_0p5()
                            .children(items),
                    ),
                    _ => this,
                })
            })
    }
}
