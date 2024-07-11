use std::rc::Rc;

use gpui::{
    actions, div, prelude::FluentBuilder, px, Action, AppContext, DismissEvent, EventEmitter,
    FocusHandle, FocusableView, InteractiveElement, KeyBinding, ParentElement, Render,
    SharedString, Styled as _, View, ViewContext, VisualContext as _, WindowContext,
};

use crate::{
    h_flex,
    list::ListItem,
    popover::{self},
    theme::ActiveTheme,
    v_flex, Icon,
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

enum PopupMenuItem {
    Separator,
    Item {
        icon: Option<Icon>,
        label: SharedString,
        handler: Rc<dyn Fn(Option<&FocusHandle>, &mut WindowContext)>,
    },
}

impl PopupMenuItem {
    fn is_clickable(&self) -> bool {
        !matches!(self, PopupMenuItem::Separator)
    }
}

pub struct PopupMenu {
    focus_handle: FocusHandle,
    action_context: Option<FocusHandle>,
    menu_items: Vec<PopupMenuItem>,
    selected_index: Option<usize>,
    _subscriptions: [gpui::Subscription; 1],
}

impl PopupMenu {
    pub fn build(
        cx: &mut WindowContext,
        f: impl FnOnce(Self, &mut WindowContext) -> Self,
    ) -> View<Self> {
        cx.new_view(|cx| {
            let focus_handle = cx.focus_handle();
            let _on_blur_subscription = cx.on_blur(&focus_handle, |this: &mut PopupMenu, cx| {
                this.dismiss(&Dismiss, cx)
            });

            let menu = Self {
                focus_handle,
                action_context: None,
                menu_items: Vec::new(),
                selected_index: None,
                _subscriptions: [_on_blur_subscription],
            };
            cx.refresh();
            f(menu, cx)
        })
    }

    /// You must set content (FocusHandle) with the parent view, if the menu action is listening on the parent view.
    /// When the Menu Item confirmed, the parent view will be focused again to ensure to receive the action.
    pub fn content(&mut self, content: FocusHandle) -> &mut Self {
        self.action_context = Some(content);
        self
    }

    /// Add Menu Item
    pub fn menu(&mut self, label: impl Into<SharedString>, action: Box<dyn Action>) -> &mut Self {
        self.add_menu_item(None, label, action)
    }

    /// Add Menu Item with Icon
    pub fn menu_with_icon(
        &mut self,
        icon: impl Into<Icon>,
        label: impl Into<SharedString>,
        action: Box<dyn Action>,
    ) -> &mut Self {
        self.add_menu_item(Some(icon.into()), label, action)
    }

    fn add_menu_item(
        &mut self,
        icon: Option<Icon>,
        label: impl Into<SharedString>,
        action: Box<dyn Action>,
    ) -> &mut Self {
        self.menu_items.push(PopupMenuItem::Item {
            icon,
            label: label.into(),
            handler: Rc::new(move |content, cx| {
                if let Some(content) = &content {
                    cx.focus(content);
                }
                cx.dispatch_action(action.boxed_clone());
            }),
        });
        self
    }

    /// Add a separator Menu Item
    pub fn separator(&mut self) -> &mut Self {
        self.menu_items.push(PopupMenuItem::Separator);

        self
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
        self.confirm(&Confirm, cx)
    }

    fn confirm(&mut self, _: &Confirm, cx: &mut ViewContext<Self>) {
        let content = self.action_context.as_ref();
        match self.selected_index {
            Some(index) => {
                let item = self.menu_items.get(index);
                match item {
                    Some(PopupMenuItem::Item { handler, .. }) => {
                        handler(content, cx);
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
        cx.emit(DismissEvent);
    }
}

impl FluentBuilder for PopupMenu {}
impl EventEmitter<DismissEvent> for PopupMenu {}
impl FocusableView for PopupMenu {
    fn focus_handle(&self, _cx: &gpui::AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for PopupMenu {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
        v_flex()
            .key_context("PopupMenu")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::select_next))
            .on_action(cx.listener(Self::select_prev))
            .on_action(cx.listener(Self::confirm))
            .on_action(cx.listener(Self::dismiss))
            .on_mouse_down_out(cx.listener(|this, _, cx| this.dismiss(&Dismiss, cx)))
            .max_h(px(550.))
            .min_w(px(230.))
            .p_1p5()
            .max_w_128()
            .gap_y_0p5()
            .children(self.menu_items.iter_mut().enumerate().map(|(ix, item)| {
                let this = ListItem::new(("menu-item", ix))
                    .on_click(cx.listener(move |this, _, cx| this.on_click(ix, cx)));
                match item {
                    PopupMenuItem::Separator => this
                        .disabled(true)
                        .child(div().h(px(1.)).m_1().border_0().bg(cx.theme().border)),
                    PopupMenuItem::Item { icon, label, .. } => {
                        this.py_1().px_2().text_sm().rounded(px(4.)).child(
                            h_flex()
                                .size_full()
                                .gap_2()
                                .items_center()
                                .children(icon.clone())
                                .child(label.clone()),
                        )
                    }
                }
            }))
    }
}
