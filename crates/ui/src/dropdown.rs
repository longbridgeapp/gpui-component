use std::rc::Rc;

use gpui::{
    actions, deferred, div, prelude::FluentBuilder as _, px, rems, AnyElement, AppContext,
    ClickEvent, DismissEvent, Element, ElementId, EventEmitter, FocusHandle, FocusableView,
    InteractiveElement, IntoElement, KeyBinding, LayoutId, ParentElement as _, Render,
    SharedString, StatefulInteractiveElement as _, Styled as _, View, ViewContext,
    VisualContext as _, WeakView, WindowContext,
};

actions!(dropdown, [Up, Down, Enter, Escape]);

pub fn init(cx: &mut AppContext) {
    let context = Some("Dropdown");
    cx.bind_keys([
        KeyBinding::new("up", Up, context),
        KeyBinding::new("down", Down, context),
        KeyBinding::new("enter", Enter, context),
        KeyBinding::new("escape", Escape, context),
    ])
}

use crate::{
    button::{Button, ButtonStyle},
    h_flex,
    list::{self, List, ListDelegate, ListItem},
    styled_ext::Sizeful,
    theme::ActiveTheme,
    Clickable as _, Icon, IconName, Size, StyledExt,
};

/// A trait for items that can be displayed in a dropdown.
pub trait DropdownItem {
    fn title(&self) -> &str;
    fn value(&self) -> &str;
}

impl DropdownItem for &String {
    fn title(&self) -> &str {
        self
    }

    fn value(&self) -> &str {
        self
    }
}

impl DropdownItem for &SharedString {
    fn title(&self) -> &str {
        self.as_ref()
    }

    fn value(&self) -> &str {
        self.as_ref()
    }
}

pub trait DropdownDelegate {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn get(&self, ix: usize) -> Option<impl DropdownItem>;
}

struct DropdownListDelegate<D: DropdownDelegate + 'static> {
    delegate: D,
    dropdown: WeakView<Dropdown<D>>,
    selected_index: Option<usize>,
}

impl<D> ListDelegate for DropdownListDelegate<D>
where
    D: DropdownDelegate + 'static,
{
    type Item = ListItem;

    fn items_count(&self) -> usize {
        self.delegate.len()
    }

    fn confirmed_index(&self) -> Option<usize> {
        self.selected_index
    }

    fn render_item(
        &self,
        ix: usize,
        _cx: &mut gpui::ViewContext<List<Self>>,
    ) -> Option<Self::Item> {
        let selected = self
            .selected_index
            .map_or(false, |selected_index| selected_index == ix);

        if let Some(item) = self.delegate.get(ix) {
            let list_item = ListItem::new(("list-item", ix))
                .check_icon(IconName::Check)
                .selected(selected)
                .py_1()
                .px_3()
                .child(item.title().to_string());
            Some(list_item)
        } else {
            None
        }
    }

    fn cancel(&mut self, cx: &mut ViewContext<List<Self>>) {
        if let Some(view) = self.dropdown.upgrade() {
            cx.update_view(&view, |view, _| {
                view.open = false;
            });
        }
    }

    fn confirm(&mut self, ix: Option<usize>, cx: &mut ViewContext<List<Self>>) {
        self.selected_index = ix;

        if let Some(view) = self.dropdown.upgrade() {
            cx.update_view(&view, |view, cx| {
                if let Some(ix) = self.selected_index {
                    if let Some(item) = self.delegate.get(ix) {
                        view.title = Some(item.title().to_string().into());
                        view.value = Some(item.value().to_string().into());
                    }
                }

                view.open = false;
                view.focus_handle.focus(cx);
            });
        }
    }
}

pub struct StringDropdownDelegate {
    items: Rc<Vec<SharedString>>,
}

impl DropdownDelegate for StringDropdownDelegate {
    fn len(&self) -> usize {
        self.items.len()
    }

    fn get(&self, ix: usize) -> Option<impl DropdownItem> {
        self.items.get(ix)
    }
}

pub struct Dropdown<D: DropdownDelegate + 'static> {
    id: ElementId,
    focus_handle: FocusHandle,
    list: View<List<DropdownListDelegate<D>>>,
    size: Size,
    open: bool,
    cleanable: bool,
    /// The value of the selected item.
    value: Option<SharedString>,
    title: Option<SharedString>,
}

impl<D> Dropdown<D>
where
    D: DropdownDelegate + 'static,
{
    pub fn new(
        id: impl Into<ElementId>,
        delegate: D,
        selected_index: Option<usize>,
        cx: &mut ViewContext<Self>,
    ) -> Self {
        let delegate = DropdownListDelegate {
            delegate,
            dropdown: cx.view().downgrade(),
            selected_index,
        };

        let (title, value) = if let Some(selected_index) = selected_index {
            let title: Option<SharedString> = delegate
                .delegate
                .get(selected_index)
                .map(|item| item.title().to_string().into());
            let value: Option<SharedString> = delegate
                .delegate
                .get(selected_index)
                .map(|item| item.value().to_string().into());
            (title, value)
        } else {
            (None, None)
        };

        let list = cx.new_view(|cx| List::new(delegate, cx).no_query().max_h(rems(20.)));
        Self {
            id: id.into(),
            focus_handle: cx.focus_handle(),
            list,
            size: Size::Medium,
            open: false,
            cleanable: true,
            title,
            value,
        }
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Set true to show the clear button when the input field is not empty.
    pub fn cleanable(mut self, cleanable: bool) -> Self {
        self.cleanable = cleanable;
        self
    }

    pub fn set_value(&mut self, value: impl Into<SharedString>, cx: &mut ViewContext<Self>) {
        self.value = Some(value.into());
        cx.notify();
    }

    pub fn value(&self) -> Option<SharedString> {
        self.value.clone()
    }

    fn up(&mut self, _: &Up, cx: &mut ViewContext<Self>) {
        if !self.open {
            return;
        }
        self.list.focus_handle(cx).focus(cx);
        cx.dispatch_action(Box::new(list::SelectPrev));
    }

    fn down(&mut self, _: &Down, cx: &mut ViewContext<Self>) {
        if !self.open {
            self.open = true;
        }

        self.list.focus_handle(cx).focus(cx);
        cx.dispatch_action(Box::new(list::SelectNext));
    }

    fn enter(&mut self, _: &Enter, cx: &mut ViewContext<Self>) {
        if !self.open {
            self.open = true;
            cx.notify();
        } else {
            self.list.focus_handle(cx).focus(cx);
            cx.dispatch_action(Box::new(list::Confirm));
        }
    }

    fn escape(&mut self, _: &Escape, cx: &mut ViewContext<Self>) {
        self.open = false;
        cx.notify();
    }

    fn clean(&mut self, _: &ClickEvent, cx: &mut ViewContext<Self>) {
        self.value = None;
        self.title = None;
        self.list.update(cx, |list, _| {
            list.selected_index = None;
        });
        cx.notify();
    }

    fn render_menu_content(&self, cx: &WindowContext) -> impl IntoElement {
        div()
            .absolute()
            .mt_1p5()
            .bg(cx.theme().background)
            .border_1()
            .border_color(cx.theme().input)
            .rounded(px(cx.theme().radius))
            .shadow_md()
            .track_focus(&self.list.focus_handle(cx))
            .child(self.list.clone())
            .on_mouse_down_out(|_, cx| {
                cx.dispatch_action(Box::new(Escape));
            })
    }
}

impl Dropdown<StringDropdownDelegate> {
    pub fn string_list(
        id: impl Into<ElementId>,
        items: Rc<Vec<SharedString>>,
        selected_index: Option<usize>,
        cx: &mut ViewContext<Self>,
    ) -> Self {
        let delegate = StringDropdownDelegate {
            items: items.clone(),
        };

        Self::new(id, delegate, selected_index, cx)
    }
}

impl<D> EventEmitter<DismissEvent> for Dropdown<D> where D: DropdownDelegate + 'static {}
impl<D> FocusableView for Dropdown<D>
where
    D: DropdownDelegate + 'static,
{
    fn focus_handle(&self, _cx: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl<D> Render for Dropdown<D>
where
    D: DropdownDelegate + 'static,
{
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let title = self.title.clone().unwrap_or_else(|| "Select...".into());
        let focused = self.focus_handle.is_focused(cx);
        let show_clean = self.cleanable && self.value.is_some();

        div()
            .key_context("Dropdown")
            .group(format!("dropdown-group:{}", self.id))
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::up))
            .on_action(cx.listener(Self::down))
            .on_action(cx.listener(Self::enter))
            .on_action(cx.listener(Self::escape))
            .size_full()
            .relative()
            .child(
                div()
                    .id(self.id.clone())
                    .relative()
                    .flex()
                    .w_full()
                    .items_center()
                    .justify_between()
                    .bg(cx.theme().background)
                    .border_1()
                    .border_color(cx.theme().input)
                    .rounded(px(cx.theme().radius))
                    .shadow_sm()
                    .when(focused, |this| this.outline(cx))
                    .input_px(self.size)
                    .input_py(self.size)
                    .input_h(self.size)
                    .on_click(cx.listener(|this, _, cx| {
                        this.open = !this.open;
                        cx.notify();
                    }))
                    .child(
                        h_flex()
                            .w_full()
                            .items_center()
                            .justify_between()
                            .child(div().flex_1().child(title))
                            .when(show_clean, |this| {
                                this.child(
                                    Button::new("clean-text", cx)
                                        .icon(IconName::Close)
                                        .style(ButtonStyle::Ghost)
                                        .size(px(14.))
                                        .cursor_pointer()
                                        .on_click(cx.listener(Self::clean)),
                                )
                            })
                            .when(!show_clean, |this| {
                                this.child(
                                    Icon::new(IconName::ChevronDown)
                                        .text_color(cx.theme().muted_foreground),
                                )
                            }),
                    ),
            )
            .child(DropdownMenuElement {
                id: "dropdown-menu".into(),
                dropdown: cx.view().clone(),
            })
    }
}

struct DropdownMenuElement<D: DropdownDelegate + 'static> {
    id: ElementId,
    dropdown: View<Dropdown<D>>,
}

#[derive(Default)]
struct DropdownMenuElementState {
    menu_element: Option<AnyElement>,
    layout_id: Option<LayoutId>,
}

impl<D> IntoElement for DropdownMenuElement<D>
where
    D: DropdownDelegate + 'static,
{
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl<D> Element for DropdownMenuElement<D>
where
    D: DropdownDelegate + 'static,
{
    type RequestLayoutState = DropdownMenuElementState;

    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn request_layout(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        cx: &mut gpui::WindowContext,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        cx.with_element_state(
            id.unwrap(),
            |element_state: Option<DropdownMenuElementState>, cx| {
                let state = element_state.unwrap_or_default();

                let menu = self
                    .dropdown
                    .read(cx)
                    .render_menu_content(cx)
                    .into_any_element();

                let mut element = deferred(menu).with_priority(1).into_any();
                let layout_id = element.request_layout(cx);
                (
                    (
                        layout_id,
                        DropdownMenuElementState {
                            layout_id: Some(layout_id),
                            menu_element: Some(element),
                        },
                    ),
                    state,
                )
            },
        )
    }

    fn prepaint(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        _: gpui::Bounds<gpui::Pixels>,
        request_layout: &mut DropdownMenuElementState,
        cx: &mut gpui::WindowContext,
    ) -> Self::PrepaintState {
        if self.dropdown.read(cx).open {
            if let Some(element) = &mut request_layout.menu_element {
                element.prepaint(cx);
            }

            if let Some(layout_id) = request_layout.layout_id {
                let bounds = cx.layout_bounds(layout_id);
                cx.insert_hitbox(bounds, false);
            }
        }
    }

    fn paint(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        _: gpui::Bounds<gpui::Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        _: &mut Self::PrepaintState,
        cx: &mut gpui::WindowContext,
    ) {
        if self.dropdown.read(cx).open {
            if let Some(element) = &mut request_layout.menu_element {
                element.paint(cx);
            }
        }
    }
}
