use std::rc::Rc;

use gpui::{
    actions, div, prelude::FluentBuilder, px, Action, AppContext, DismissEvent, EventEmitter,
    FocusHandle, FocusableView, InteractiveElement, KeyBinding, ParentElement, Render,
    SharedString, Styled as _, View, ViewContext, VisualContext as _, WindowContext,
};

use crate::{h_flex, list::ListItem, theme::ActiveTheme, v_flex, Icon, StyledExt as _};

actions!(menu, [Confirm, Cancel, SelectNext, SelectPrev]);

pub fn init(cx: &mut AppContext) {
    let context = Some("PopupMenu");
    cx.bind_keys([
        KeyBinding::new("enter", Confirm, context),
        KeyBinding::new("escape", Cancel, context),
        KeyBinding::new("up", SelectPrev, context),
        KeyBinding::new("down", SelectNext, context),
    ]);
}

enum PopupMenuItem {
    Separator,
    Label(SharedString),
    Item {
        icon: Option<Icon>,
        label: SharedString,
        action: Option<Box<dyn Action>>,
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
    menu_items: Vec<PopupMenuItem>,
    selected_index: Option<usize>,
}

impl PopupMenu {
    pub fn build(
        cx: &mut WindowContext,
        f: impl FnOnce(Self, &mut WindowContext) -> Self,
    ) -> View<Self> {
        cx.new_view(|cx| {
            let menu = Self {
                focus_handle: cx.focus_handle(),
                menu_items: Vec::new(),
                selected_index: None,
            };

            f(menu, cx)
        })
    }

    /// Add a simple label Menu Item
    pub fn label(&mut self, label: impl Into<SharedString>) -> &mut Self {
        self.menu_items.push(PopupMenuItem::Label(label.into()));
        self
    }

    /// Add Menu Item
    pub fn menu(&mut self, label: impl Into<SharedString>, action: Box<dyn Action>) -> &mut Self {
        self.menu_items.push(PopupMenuItem::Item {
            icon: None,
            label: label.into(),
            action: Some(action.boxed_clone()),
            handler: Rc::new(move |context, cx| {
                if let Some(context) = &context {
                    cx.focus(context);
                }
                cx.dispatch_action(action.boxed_clone());
            }),
        });
        self
    }

    /// Add Menu Item with Icon
    pub fn menu_with_icon(
        &mut self,
        label: impl Into<SharedString>,
        icon: impl Into<Icon>,
        action: Box<dyn Action>,
    ) -> &mut Self {
        self.menu_items.push(PopupMenuItem::Item {
            icon: Some(icon.into()),
            label: label.into(),
            action: Some(action.boxed_clone()),
            handler: Rc::new(move |context, cx| {
                if let Some(context) = &context {
                    cx.focus(context);
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
        match self.selected_index {
            Some(index) => {
                let item = self.menu_items.get(index);
                match item {
                    Some(PopupMenuItem::Item { handler, .. }) => {
                        handler(Some(&cx.focus_handle()), cx);
                        cx.emit(DismissEvent);
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

    fn cancel(&mut self, _: &Cancel, cx: &mut ViewContext<Self>) {
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
            .on_action(cx.listener(Self::cancel))
            .on_mouse_down_out(cx.listener(|this, _, cx| this.cancel(&Cancel, cx)))
            .max_h(px(550.))
            .min_w(px(230.))
            .p_1p5()
            .max_w_128()
            .gap_y_0p5()
            .children(self.menu_items.iter_mut().enumerate().map(|(ix, item)| {
                let this = ListItem::new(("menu-item", ix))
                    .py_1()
                    .px_2()
                    .text_sm()
                    .rounded(px(4.))
                    .on_click(cx.listener(move |this, _, cx| this.on_click(ix, cx)));
                match item {
                    PopupMenuItem::Separator => this
                        .disabled(true)
                        .child(div().h(px(1.)).bg(cx.theme().border)),
                    PopupMenuItem::Label(label) => this.child(label.clone()),
                    PopupMenuItem::Item { icon, label, .. } => this.child(
                        h_flex()
                            .size_full()
                            .gap_1()
                            .items_center()
                            .when_some(icon.clone(), |this, icon| this.child(icon))
                            .child(label.clone()),
                    ),
                }
            }))
    }
}
