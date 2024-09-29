use std::ops::Deref;
use std::rc::Rc;

use gpui::{
    actions, div, prelude::FluentBuilder, px, Action, AppContext, DismissEvent, EventEmitter,
    FocusHandle, InteractiveElement, IntoElement, KeyBinding, ParentElement, Pixels, Render,
    SharedString, Styled as _, View, ViewContext, VisualContext as _, WindowContext,
};
use gpui::{anchored, canvas, rems, AnchorCorner, Bounds, FocusableView, Keystroke, WeakView};

use crate::StyledExt;
use crate::{
    button::Button, h_flex, list::ListItem, popover::Popover, theme::ActiveTheme, v_flex, Icon,
    IconName, Selectable, Sizable as _,
};

actions!(menu, [Confirm, Dismiss, SelectNext, SelectPrev]);

pub fn init(cx: &mut AppContext) {
    let context = Some("PopupMenu");
    cx.bind_keys([
        KeyBinding::new("enter", Confirm, context),
        KeyBinding::new("escape", Dismiss, context),
        KeyBinding::new("up", SelectPrev, context),
        KeyBinding::new("down", SelectNext, context),
    ]);
}

pub trait PopupMenuExt: Selectable + IntoElement + 'static {
    fn popup_menu(
        self,
        f: impl Fn(PopupMenu, &mut ViewContext<PopupMenu>) -> PopupMenu + 'static,
    ) -> Popover<PopupMenu> {
        Popover::new("popup-menu")
            .no_style()
            .trigger(self)
            .content(move |cx| PopupMenu::build(cx, |menu, cx| f(menu, cx)))
    }
}
impl PopupMenuExt for Button {}

enum PopupMenuItem {
    Separator,
    Item {
        icon: Option<Icon>,
        label: SharedString,
        action: Option<Box<dyn Action>>,
        handler: Rc<dyn Fn(&mut WindowContext)>,
    },
    Submenu {
        icon: Option<Icon>,
        label: SharedString,
        menu: View<PopupMenu>,
    },
}

impl PopupMenuItem {
    fn is_clickable(&self) -> bool {
        !matches!(self, PopupMenuItem::Separator)
    }

    fn is_separator(&self) -> bool {
        matches!(self, PopupMenuItem::Separator)
    }

    fn has_icon(&self) -> bool {
        matches!(self, PopupMenuItem::Item { icon: Some(_), .. })
    }
}

pub struct PopupMenu {
    /// The parent menu of this menu, if this is a submenu
    parent_menu: Option<WeakView<Self>>,
    focus_handle: FocusHandle,
    menu_items: Vec<PopupMenuItem>,
    has_icon: bool,
    selected_index: Option<usize>,
    min_width: Pixels,
    max_width: Pixels,
    hovered_menu_ix: Option<usize>,
    bounds: Bounds<Pixels>,

    action_focus_handle: Option<FocusHandle>,
    _subscriptions: [gpui::Subscription; 1],
}

impl PopupMenu {
    pub fn build(
        cx: &mut WindowContext,
        f: impl FnOnce(Self, &mut ViewContext<PopupMenu>) -> Self,
    ) -> View<Self> {
        cx.new_view(|cx| {
            let focus_handle = cx.focus_handle();
            let _on_blur_subscription = cx.on_blur(&focus_handle, |this: &mut PopupMenu, cx| {
                this.dismiss(&Dismiss, cx)
            });

            let menu = Self {
                focus_handle,
                action_focus_handle: None,
                parent_menu: None,
                menu_items: Vec::new(),
                selected_index: None,
                min_width: px(120.),
                max_width: px(500.),
                has_icon: false,
                hovered_menu_ix: None,
                bounds: Bounds::default(),
                _subscriptions: [_on_blur_subscription],
            };
            cx.refresh();
            f(menu, cx)
        })
    }

    /// Bind the focus handle of the menu, when clicked, it will focus back to this handle and then dispath the action
    pub fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.action_focus_handle = Some(focus_handle.clone());
        self
    }

    /// Set min width of the popup menu, default is 120px
    pub fn min_w(mut self, width: impl Into<Pixels>) -> Self {
        self.min_width = width.into();
        self
    }

    /// Set max width of the popup menu, default is 500px
    pub fn max_w(mut self, width: impl Into<Pixels>) -> Self {
        self.max_width = width.into();
        self
    }

    /// Add Menu Item
    pub fn menu(mut self, label: impl Into<SharedString>, action: Box<dyn Action>) -> Self {
        self.add_menu_item(label, None, action);
        self
    }

    /// Add Menu to open link
    pub fn link(mut self, label: impl Into<SharedString>, href: impl Into<String>) -> Self {
        let href = href.into();
        self.menu_items.push(PopupMenuItem::Item {
            icon: None,
            label: label.into(),
            action: None,
            handler: Rc::new(move |cx| cx.open_url(&href)),
        });
        self
    }

    /// Add Menu to open link
    pub fn link_with_icon(
        mut self,
        label: impl Into<SharedString>,
        icon: impl Into<Icon>,
        href: impl Into<String>,
    ) -> Self {
        let href = href.into();
        self.menu_items.push(PopupMenuItem::Item {
            icon: Some(icon.into()),
            label: label.into(),
            action: None,
            handler: Rc::new(move |cx| cx.open_url(&href)),
        });
        self
    }

    /// Add Menu Item with Icon
    pub fn menu_with_icon(
        mut self,
        label: impl Into<SharedString>,
        icon: impl Into<Icon>,
        action: Box<dyn Action>,
    ) -> Self {
        self.add_menu_item(label, Some(icon.into()), action);
        self
    }

    /// Add Menu Item with check icon
    pub fn menu_with_check(
        mut self,
        label: impl Into<SharedString>,
        checked: bool,
        action: Box<dyn Action>,
    ) -> Self {
        if checked {
            self.add_menu_item(label, Some(IconName::Check.into()), action);
        } else {
            self.add_menu_item(label, None, action);
        }

        self
    }

    fn add_menu_item(
        &mut self,
        label: impl Into<SharedString>,
        icon: Option<Icon>,
        action: Box<dyn Action>,
    ) -> &mut Self {
        if icon.is_some() {
            self.has_icon = true;
        }

        let action_focus_handle = self.action_focus_handle.clone();

        self.menu_items.push(PopupMenuItem::Item {
            icon,
            label: label.into(),
            action: Some(action.boxed_clone()),
            handler: Rc::new(move |cx| {
                cx.activate_window();

                // Focus back to the user expected focus handle
                // Then the actions listened on that focus handle can be received
                //
                // For example:
                //
                // TabPanel
                //   |- PopupMenu
                //   |- PanelContent (actions are listened here)
                //
                // The `PopupMenu` and `PanelContent` are at the same level in the TabPanel
                // If the actions are listened on the `PanelContent`,
                // it can't receive the actions from the `PopupMenu`, unless we focus on `PanelContent`.
                if let Some(handle) = action_focus_handle.as_ref() {
                    cx.focus(&handle);
                }

                cx.dispatch_action(action.boxed_clone());
            }),
        });
        self
    }

    /// Add a separator Menu Item
    pub fn separator(mut self) -> Self {
        if self.menu_items.is_empty() {
            return self;
        }

        if let Some(PopupMenuItem::Separator) = self.menu_items.last() {
            return self;
        }

        self.menu_items.push(PopupMenuItem::Separator);
        self
    }

    pub fn submenu(
        self,
        label: impl Into<SharedString>,
        cx: &mut ViewContext<Self>,
        f: impl Fn(PopupMenu, &mut ViewContext<PopupMenu>) -> PopupMenu + 'static,
    ) -> Self {
        self.submenu_with_icon(None, label, cx, f)
    }

    /// Add a Submenu item with icon
    pub fn submenu_with_icon(
        mut self,
        icon: Option<Icon>,
        label: impl Into<SharedString>,
        cx: &mut ViewContext<Self>,
        f: impl Fn(PopupMenu, &mut ViewContext<PopupMenu>) -> PopupMenu + 'static,
    ) -> Self {
        let submenu = PopupMenu::build(cx, f);
        let parent_menu = cx.view().downgrade();
        submenu.update(cx, |view, _| {
            view.parent_menu = Some(parent_menu);
        });

        self.menu_items.push(PopupMenuItem::Submenu {
            icon,
            label: label.into(),
            menu: submenu,
        });
        self
    }

    pub(crate) fn active_submenu(&self) -> Option<View<PopupMenu>> {
        if let Some(ix) = self.hovered_menu_ix {
            if let Some(item) = self.menu_items.get(ix) {
                return match item {
                    PopupMenuItem::Submenu { menu, .. } => Some(menu.clone()),
                    _ => None,
                };
            }
        }

        None
    }

    fn clickable_menu_items(&self) -> impl Iterator<Item = (usize, &PopupMenuItem)> {
        self.menu_items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.is_clickable())
    }

    fn on_click(&mut self, ix: usize, cx: &mut ViewContext<Self>) {
        cx.stop_propagation();
        cx.prevent_default();
        self.selected_index = Some(ix);
        self.confirm(&Confirm, cx);
    }

    fn confirm(&mut self, _: &Confirm, cx: &mut ViewContext<Self>) {
        match self.selected_index {
            Some(index) => {
                let item = self.menu_items.get(index);
                match item {
                    Some(PopupMenuItem::Item { handler, .. }) => {
                        handler(cx);
                        self.dismiss(&Dismiss, cx)
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn select_next(&mut self, _: &SelectNext, cx: &mut ViewContext<Self>) {
        let count = self.clickable_menu_items().count();
        if count > 0 {
            let ix = self
                .selected_index
                .map(|index| if index == count - 1 { 0 } else { index + 1 })
                .unwrap_or(0);

            self.selected_index = Some(ix);
            cx.notify();
        }
    }

    fn select_prev(&mut self, _: &SelectPrev, cx: &mut ViewContext<Self>) {
        let count = self.clickable_menu_items().count();
        if count > 0 {
            let ix = self
                .selected_index
                .map(|index| if index == count - 1 { 0 } else { index - 1 })
                .unwrap_or(count - 1);
            self.selected_index = Some(ix);
            cx.notify();
        }
    }

    fn dismiss(&mut self, _: &Dismiss, cx: &mut ViewContext<Self>) {
        if self.active_submenu().is_some() {
            return;
        }

        cx.emit(DismissEvent);
        // Dismiss parent menu, when this menu is dismissed
        if let Some(parent_menu) = self.parent_menu.clone().and_then(|menu| menu.upgrade()) {
            parent_menu.update(cx, |view, cx| {
                view.hovered_menu_ix = None;
                view.dismiss(&Dismiss, cx);
            })
        }
    }

    fn render_keybinding(
        action: Option<Box<dyn Action>>,
        cx: &ViewContext<Self>,
    ) -> Option<impl IntoElement> {
        if let Some(action) = action {
            if let Some(keybinding) = cx.bindings_for_action(action.deref()).first() {
                let el = div().text_color(cx.theme().muted_foreground).children(
                    keybinding
                        .keystrokes()
                        .into_iter()
                        .map(|key| key_shortcut(key.clone())),
                );

                return Some(el);
            }
        }

        return None;
    }

    fn render_icon(
        has_icon: bool,
        icon: Option<Icon>,
        _: &ViewContext<Self>,
    ) -> Option<impl IntoElement> {
        let icon_placeholder = if has_icon { Some(Icon::empty()) } else { None };

        if !has_icon {
            return None;
        }

        let icon = h_flex()
            .w_3p5()
            .h_3p5()
            .items_center()
            .justify_center()
            .text_sm()
            .map(|this| {
                if let Some(icon) = icon {
                    this.child(icon.clone().small().clone())
                } else {
                    this.children(icon_placeholder.clone())
                }
            });

        Some(icon)
    }
}

impl FluentBuilder for PopupMenu {}
impl EventEmitter<DismissEvent> for PopupMenu {}
impl FocusableView for PopupMenu {
    fn focus_handle(&self, _: &gpui::AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for PopupMenu {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
        let view = cx.view().clone();
        let has_icon = self.menu_items.iter().any(|item| item.has_icon());
        let items_count = self.menu_items.len();
        let max_width = self.max_width;
        let bounds = self.bounds;

        v_flex()
            .key_context("PopupMenu")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::select_next))
            .on_action(cx.listener(Self::select_prev))
            .on_action(cx.listener(Self::confirm))
            .on_action(cx.listener(Self::dismiss))
            .on_mouse_down_out(cx.listener(|this, _, cx| this.dismiss(&Dismiss, cx)))
            .max_h(self.max_width)
            .min_w(self.min_width)
            .p_1()
            .gap_y_0p5()
            .min_w(rems(8.))
            .popover_style(cx)
            .text_color(cx.theme().popover_foreground)
            .relative()
            .child({
                canvas(
                    move |bounds, cx| view.update(cx, |r, _| r.bounds = bounds),
                    |_, _, _| {},
                )
                .absolute()
                .size_full()
            })
            .children(
                self.menu_items
                    .iter_mut()
                    .enumerate()
                    // Skip last separator
                    .filter(|(ix, item)| !(*ix == items_count - 1 && item.is_separator()))
                    .map(|(ix, item)| {
                        let group_id = format!("item:{}", ix);

                        let this = ListItem::new(("menu-item", ix))
                            .group(group_id.clone())
                            .relative()
                            .text_sm()
                            .py_0()
                            .px_2()
                            .h(px(28.))
                            .rounded_md()
                            .items_center()
                            .on_mouse_enter(cx.listener(move |this, _, cx| {
                                this.hovered_menu_ix = Some(ix);
                                cx.notify();
                            }));

                        match item {
                            PopupMenuItem::Separator => this.h_auto().p_0().disabled(true).child(
                                div()
                                    .rounded_none()
                                    .h(px(1.))
                                    .mx_neg_1()
                                    .my_0p5()
                                    .bg(cx.theme().muted),
                            ),
                            PopupMenuItem::Item {
                                icon,
                                label,
                                action,
                                ..
                            } => {
                                let action = action.as_ref().map(|action| action.boxed_clone());
                                let key = Self::render_keybinding(action, cx);

                                this.on_click(cx.listener(move |this, _, cx| this.on_click(ix, cx)))
                                    .child(
                                        h_flex()
                                            .items_center()
                                            .gap_x_1p5()
                                            .children(Self::render_icon(has_icon, icon.clone(), cx))
                                            .child(
                                                h_flex()
                                                    .flex_1()
                                                    .gap_2()
                                                    .items_center()
                                                    .justify_between()
                                                    .child(label.clone())
                                                    .children(key),
                                            ),
                                    )
                            }
                            PopupMenuItem::Submenu { icon, label, menu } => this
                                .when(self.hovered_menu_ix == Some(ix), |this| this.selected(true))
                                .child(
                                    h_flex()
                                        .items_start()
                                        .child(
                                            h_flex()
                                                .size_full()
                                                .items_center()
                                                .gap_x_1p5()
                                                .children(Self::render_icon(
                                                    has_icon,
                                                    icon.clone(),
                                                    cx,
                                                ))
                                                .child(
                                                    h_flex()
                                                        .flex_1()
                                                        .gap_2()
                                                        .items_center()
                                                        .justify_between()
                                                        .child(label.clone())
                                                        .child(IconName::ChevronRight),
                                                ),
                                        )
                                        .when_some(self.hovered_menu_ix, |this, hovered_ix| {
                                            let (anchor, left) = if cx.bounds().size.width
                                                - bounds.origin.x
                                                < max_width
                                            {
                                                (AnchorCorner::TopRight, -px(15.))
                                            } else {
                                                (AnchorCorner::TopLeft, bounds.size.width - px(10.))
                                            };

                                            let top = if bounds.origin.y + bounds.size.height
                                                > cx.bounds().size.height
                                            {
                                                px(32.)
                                            } else {
                                                -px(10.)
                                            };

                                            if hovered_ix == ix {
                                                this.child(
                                                    anchored().anchor(anchor).child(
                                                        div()
                                                            .occlude()
                                                            .top(top)
                                                            .left(left)
                                                            .child(menu.clone()),
                                                    ),
                                                )
                                            } else {
                                                this
                                            }
                                        }),
                                ),
                        }
                    }),
            )
    }
}

/// Return the Platform specific keybinding string by KeyStroke
pub fn key_shortcut(key: Keystroke) -> String {
    if cfg!(target_os = "macos") {
        return format!("{}", key);
    }

    let mut parts = vec![];
    if key.modifiers.control {
        parts.push("Ctrl");
    }
    if key.modifiers.alt {
        parts.push("Alt");
    }
    if key.modifiers.platform {
        parts.push("Win");
    }
    if key.modifiers.shift {
        parts.push("Shift");
    }

    // Capitalize the first letter
    let key = if let Some(first_c) = key.key.chars().next() {
        format!("{}{}", first_c.to_uppercase(), &key.key[1..])
    } else {
        key.key.to_string()
    };

    parts.push(&key);
    parts.join("+")
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_key_shortcut() {
        use super::key_shortcut;
        use gpui::Keystroke;

        if cfg!(target_os = "windows") {
            assert_eq!(key_shortcut(Keystroke::parse("a").unwrap()), "A");
            assert_eq!(key_shortcut(Keystroke::parse("ctrl-a").unwrap()), "Ctrl+A");
            assert_eq!(
                key_shortcut(Keystroke::parse("ctrl-alt-a").unwrap()),
                "Ctrl+Alt+A"
            );
            assert_eq!(
                key_shortcut(Keystroke::parse("ctrl-alt-shift-a").unwrap()),
                "Ctrl+Alt+Shift+A"
            );
            assert_eq!(
                key_shortcut(Keystroke::parse("ctrl-alt-shift-win-a").unwrap()),
                "Ctrl+Alt+Win+Shift+A"
            );
            assert_eq!(
                key_shortcut(Keystroke::parse("ctrl-shift-backspace").unwrap()),
                "Ctrl+Shift+Backspace"
            );
        }
    }
}
