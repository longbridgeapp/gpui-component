use gpui::{
    actions, deferred, div, prelude::FluentBuilder, px, rems, AnyElement, AppContext, ClickEvent,
    DismissEvent, Div, Element, ElementId, EventEmitter, FocusHandle, Focusable, FocusableView,
    InteractiveElement, IntoElement, KeyBinding, LayoutId, Length, ParentElement, Render,
    SharedString, StatefulInteractiveElement, Styled, Task, View, ViewContext, VisualContext,
    WeakView, WindowContext,
};

use crate::{
    button::{Button, ButtonStyle},
    h_flex,
    list::{self, List, ListDelegate, ListItem},
    styled_ext::StyleSized,
    theme::ActiveTheme,
    Clickable, Icon, IconName, Size, StyledExt,
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

/// A trait for items that can be displayed in a dropdown.
pub trait DropdownItem {
    type Value: Clone;
    fn title(&self) -> SharedString;
    fn value(&self) -> &Self::Value;
}

impl DropdownItem for String {
    type Value = Self;

    fn title(&self) -> SharedString {
        SharedString::from(self.to_string())
    }

    fn value(&self) -> &Self::Value {
        &self
    }
}

impl DropdownItem for SharedString {
    type Value = Self;

    fn title(&self) -> SharedString {
        SharedString::from(self.to_string())
    }

    fn value(&self) -> &Self::Value {
        &self
    }
}

pub trait DropdownDelegate: Sized {
    type Item: DropdownItem;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn get(&self, ix: usize) -> Option<&Self::Item>;

    fn position<V>(&self, value: &V) -> Option<usize>
    where
        Self::Item: DropdownItem<Value = V>,
        V: PartialEq,
    {
        (0..self.len()).find(|&i| self.get(i).map_or(false, |item| item.value() == value))
    }

    fn can_search(&self) -> bool {
        false
    }

    fn perform_search(&mut self, _query: &str, _cx: &mut ViewContext<Dropdown<Self>>) -> Task<()> {
        Task::Ready(Some(()))
    }
}

impl<T: DropdownItem> DropdownDelegate for Vec<T> {
    type Item = T;

    fn len(&self) -> usize {
        self.len()
    }

    fn get(&self, ix: usize) -> Option<&Self::Item> {
        self.as_slice().get(ix)
    }

    fn position<V>(&self, value: &V) -> Option<usize>
    where
        Self::Item: DropdownItem<Value = V>,
        V: PartialEq,
    {
        self.iter().position(|v| v.value() == value)
    }
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

    fn render_item(&self, ix: usize, cx: &mut gpui::ViewContext<List<Self>>) -> Option<Self::Item> {
        let selected = self
            .selected_index
            .map_or(false, |selected_index| selected_index == ix);
        let size = self
            .dropdown
            .upgrade()
            .map_or(Size::Medium, |dropdown| dropdown.read(cx).size);

        if let Some(item) = self.delegate.get(ix) {
            let list_item = ListItem::new(("list-item", ix))
                .check_icon(IconName::Check)
                .selected(selected)
                .input_text_size(size)
                .list_size(size)
                .child(div().whitespace_nowrap().child(item.title().to_string()));
            Some(list_item)
        } else {
            None
        }
    }

    fn cancel(&mut self, cx: &mut ViewContext<List<Self>>) {
        if let Some(view) = self.dropdown.upgrade() {
            cx.update_view(&view, |view, cx| {
                view.focus(cx);
                view.open = false;
            });
        }
    }

    fn confirm(&mut self, ix: Option<usize>, cx: &mut ViewContext<List<Self>>) {
        self.selected_index = ix;

        if let Some(view) = self.dropdown.upgrade() {
            cx.update_view(&view, |view, cx| {
                let selected_value = self
                    .selected_index
                    .and_then(|ix| self.delegate.get(ix))
                    .map(|item| item.value().clone());
                cx.emit(DropdownEvent::Confirm(selected_value.clone()));
                view.focus(cx);
                view.selected_value = selected_value;
                view.open = false;
            });
        }
    }

    fn perform_search(&mut self, query: &str, cx: &mut ViewContext<List<Self>>) -> Task<()> {
        self.dropdown
            .upgrade()
            .map_or(Task::Ready(None), |dropdown| {
                dropdown.update(cx, |_, cx| self.delegate.perform_search(query, cx))
            })
    }

    fn set_selected_index(&mut self, ix: Option<usize>, _: &mut ViewContext<List<Self>>) {
        self.selected_index = ix;
    }

    fn render_empty(&self, cx: &mut ViewContext<List<Self>>) -> impl IntoElement {
        if let Some(empty) = self
            .dropdown
            .upgrade()
            .and_then(|dropdown| dropdown.read(cx).empty.as_ref())
        {
            empty(cx).into_any_element()
        } else {
            h_flex()
                .justify_center()
                .py_6()
                .text_color(cx.theme().muted)
                .child(Icon::new(IconName::Inbox).size(px(28.)))
                .into_any_element()
        }
    }
}

pub enum DropdownEvent<D: DropdownDelegate + 'static> {
    Confirm(Option<<D::Item as DropdownItem>::Value>),
}

pub struct Dropdown<D: DropdownDelegate + 'static> {
    id: ElementId,
    focus_handle: FocusHandle,
    list: View<List<DropdownListDelegate<D>>>,
    size: Size,
    open: bool,
    cleanable: bool,
    placeholder: SharedString,
    title_prefix: Option<SharedString>,
    selected_value: Option<<D::Item as DropdownItem>::Value>,
    empty: Option<Box<dyn Fn(&WindowContext) -> AnyElement + 'static>>,
    width: Length,
    menu_width: Length,
}

pub struct SearchableVec<T> {
    items: Vec<T>,
    matched_items: Vec<T>,
}

impl<T: DropdownItem + Clone> SearchableVec<T> {
    pub fn new(items: impl Into<Vec<T>>) -> Self {
        let items = items.into();
        Self {
            items: items.clone(),
            matched_items: items,
        }
    }
}

impl<T: DropdownItem + Clone> DropdownDelegate for SearchableVec<T> {
    type Item = T;

    fn len(&self) -> usize {
        self.matched_items.len()
    }

    fn get(&self, ix: usize) -> Option<&Self::Item> {
        self.matched_items.get(ix)
    }

    fn position<V>(&self, value: &V) -> Option<usize>
    where
        Self::Item: DropdownItem<Value = V>,
        V: PartialEq,
    {
        for (ix, item) in self.matched_items.iter().enumerate() {
            if item.value() == value {
                return Some(ix);
            }
        }

        None
    }

    fn can_search(&self) -> bool {
        true
    }

    fn perform_search(&mut self, query: &str, _cx: &mut ViewContext<Dropdown<Self>>) -> Task<()> {
        self.matched_items = self
            .items
            .iter()
            .filter(|item| item.title().to_lowercase().contains(&query.to_lowercase()))
            .cloned()
            .collect();

        Task::Ready(Some(()))
    }
}

impl From<Vec<SharedString>> for SearchableVec<SharedString> {
    fn from(items: Vec<SharedString>) -> Self {
        Self {
            items: items.clone(),
            matched_items: items,
        }
    }
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

        let searchable = delegate.delegate.can_search();

        let list = cx.new_view(|cx| {
            let mut list = List::new(delegate, cx).max_h(rems(20.));
            if !searchable {
                list = list.no_query();
            }
            list
        });
        let mut this = Self {
            id: id.into(),
            focus_handle: cx.focus_handle(),
            placeholder: "Select...".into(),
            list,
            size: Size::Medium,
            selected_value: None,
            open: false,
            cleanable: false,
            title_prefix: None,
            empty: None,
            width: Length::Auto,
            menu_width: Length::Auto,
        };
        this.set_selected_index(selected_index, cx);
        this
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Set the width of the dropdown input, default: Length::Auto
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Set the width of the dropdown menu, default: Length::Auto
    pub fn menu_width(mut self, width: impl Into<Length>) -> Self {
        self.menu_width = width.into();
        self
    }

    /// Set the placeholder for display when dropdown value is empty.
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Set title prefix for the dropdown.
    ///
    /// e.g.: Country: United States
    ///
    /// You should set the label is `Country: `
    pub fn title_prefix(mut self, prefix: impl Into<SharedString>) -> Self {
        self.title_prefix = Some(prefix.into());
        self
    }

    /// Set true to show the clear button when the input field is not empty.
    pub fn cleanable(mut self, cleanable: bool) -> Self {
        self.cleanable = cleanable;
        self
    }

    pub fn empty<E, F>(mut self, f: F) -> Self
    where
        E: IntoElement,
        F: Fn(&WindowContext) -> E + 'static,
    {
        self.empty = Some(Box::new(move |cx| f(cx).into_any_element()));
        self
    }

    pub fn set_selected_index(
        &mut self,
        selected_index: Option<usize>,
        cx: &mut ViewContext<Self>,
    ) {
        self.list.update(cx, |list, cx| {
            list.set_selected_index(selected_index, cx);
        });
        self.update_selected_value(cx);
    }

    pub fn set_selected_value(
        &mut self,
        selected_value: &<D::Item as DropdownItem>::Value,
        cx: &mut ViewContext<Self>,
    ) where
        <<D as DropdownDelegate>::Item as DropdownItem>::Value: PartialEq,
    {
        let delegate = self.list.read(cx).delegate();
        let selected_index = delegate.delegate.position(selected_value);
        self.set_selected_index(selected_index, cx);
    }

    pub fn selected_index(&self, cx: &WindowContext) -> Option<usize> {
        self.list.read(cx).selected_index()
    }

    fn update_selected_value(&mut self, cx: &WindowContext) {
        self.selected_value = self
            .selected_index(cx)
            .and_then(|ix| self.list.read(cx).delegate().delegate.get(ix))
            .map(|item| item.value().clone());
    }

    pub fn selected_value(&self) -> Option<&<D::Item as DropdownItem>::Value> {
        self.selected_value.as_ref()
    }

    pub fn focus(&self, cx: &mut WindowContext) {
        self.focus_handle.focus(cx);
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

    fn toggle_menu(&mut self, _: &ClickEvent, cx: &mut ViewContext<Self>) {
        self.open = !self.open;
        if self.open {
            self.list.focus_handle(cx).focus(cx);
        }
        cx.notify();
    }

    fn escape(&mut self, _: &Escape, cx: &mut ViewContext<Self>) {
        self.open = false;
        cx.notify();
    }

    fn clean(&mut self, _: &ClickEvent, cx: &mut ViewContext<Self>) {
        self.set_selected_index(None, cx)
    }

    fn render_menu_content(&self, cx: &WindowContext) -> impl IntoElement {
        div()
            .track_focus(&self.list.focus_handle(cx))
            .on_mouse_down_out(|_, cx| {
                cx.dispatch_action(Box::new(Escape));
            })
            .map(|this| with_style(this, cx).child(self.list.clone()))
    }

    fn display_title(&self, cx: &WindowContext) -> impl IntoElement {
        if let Some(selected_index) = &self.selected_index(cx) {
            let title = self
                .list
                .read(cx)
                .delegate()
                .delegate
                .get(*selected_index)
                .map(|item| item.title().to_string())
                .unwrap_or_default();

            h_flex()
                .children(self.title_prefix.clone().map(|prefix| {
                    div()
                        .text_color(cx.theme().accent_foreground)
                        .child(prefix.clone())
                }))
                .child(title.clone())
        } else {
            div()
                .text_color(cx.theme().accent_foreground)
                .child(self.placeholder.clone())
        }
    }
}

fn with_style(d: Focusable<Div>, cx: &WindowContext) -> Focusable<Div> {
    d.absolute()
        .mt_1p5()
        .bg(cx.theme().background)
        .border_1()
        .border_color(cx.theme().input)
        .rounded(px(cx.theme().radius))
        .shadow_md()
}

impl<D> EventEmitter<DropdownEvent<D>> for Dropdown<D> where D: DropdownDelegate + 'static {}
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
        let is_focused = self.focus_handle.is_focused(cx);
        let show_clean = self.cleanable && self.selected_index(cx).is_some();

        div()
            .id(self.id.clone())
            .key_context("Dropdown")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::up))
            .on_action(cx.listener(Self::down))
            .on_action(cx.listener(Self::enter))
            .on_action(cx.listener(Self::escape))
            .size_full()
            .relative()
            .input_text_size(self.size)
            .map(|this| match self.width {
                Length::Definite(l) => this.flex_none().w(l),
                Length::Auto => this.w_full(),
            })
            .child(
                div()
                    .id("dropdown-input")
                    .relative()
                    .flex()
                    .items_center()
                    .justify_between()
                    .bg(cx.theme().background)
                    .border_1()
                    .border_color(cx.theme().input)
                    .rounded(px(cx.theme().radius))
                    .shadow_sm()
                    .overflow_hidden()
                    .input_text_size(self.size)
                    .when(is_focused, |this| this.outline(cx))
                    .input_size(self.size)
                    .when(!self.open, |this| {
                        this.on_click(cx.listener(Self::toggle_menu))
                    })
                    .child(
                        h_flex()
                            .w_full()
                            .items_center()
                            .justify_between()
                            .gap_1()
                            .child(
                                div()
                                    .w_full()
                                    .overflow_hidden()
                                    .child(self.display_title(cx)),
                            )
                            .when(show_clean, |this| {
                                this.child(
                                    Button::new("clean", cx)
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
            .child(
                div()
                    .absolute()
                    .overflow_hidden()
                    .map(|this| match self.menu_width {
                        Length::Auto => this,
                        Length::Definite(l) => this.w(l),
                    })
                    .child(DropdownMenuElement {
                        id: "dropdown-menu".into(),
                        dropdown: cx.view().clone(),
                    }),
            )
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
