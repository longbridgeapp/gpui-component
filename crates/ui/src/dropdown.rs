use gpui::{
    actions, deferred, div, prelude::FluentBuilder as _, px, AnyElement, AppContext, DismissEvent,
    Element, ElementId, EventEmitter, FocusHandle, FocusableView, InteractiveElement, IntoElement,
    KeyBinding, LayoutId, ParentElement as _, Render, SharedString,
    StatefulInteractiveElement as _, Styled as _, View, ViewContext, VisualContext as _, WeakView,
    WindowContext,
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
    h_flex,
    list::ListItem,
    picker::{self, Picker, PickerDelegate},
    theme::ActiveTheme,
    Icon, IconName,
};

/// A trait for items that can be displayed in a dropdown.
pub trait DropdownItem {
    fn title(&self) -> &str;
    fn value(&self) -> &str;
}

impl DropdownItem for String {
    fn title(&self) -> &str {
        self
    }

    fn value(&self) -> &str {
        self
    }
}

pub trait DropdownDelegate {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn get(&self, ix: usize) -> Option<&dyn DropdownItem>;
}

struct DropdownPickerDelegate<D: DropdownDelegate + 'static> {
    delegate: D,
    dropdown: WeakView<Dropdown<D>>,
    selected_index: usize,
}

impl<D> PickerDelegate for DropdownPickerDelegate<D>
where
    D: DropdownDelegate + 'static,
{
    type ListItem = ListItem;

    fn match_count(&self) -> usize {
        self.delegate.len()
    }

    fn selected_index(&self) -> usize {
        self.selected_index
    }

    fn set_selected_index(&mut self, index: usize, _cx: &mut gpui::ViewContext<Picker<Self>>) {
        self.selected_index = index;
    }

    fn render_item(
        &self,
        ix: usize,
        selected: bool,
        _cx: &mut gpui::ViewContext<Picker<Self>>,
    ) -> Option<Self::ListItem> {
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

    fn dismissed(&mut self, cx: &mut ViewContext<Picker<Self>>) {
        if let Some(view) = self.dropdown.upgrade() {
            cx.update_view(&view, |view, _| {
                view.open = false;
            });
        }
    }

    fn confirm(&mut self, _secondary: bool, cx: &mut ViewContext<Picker<Self>>) {
        if let Some(view) = self.dropdown.upgrade() {
            cx.update_view(&view, |view, cx| {
                if let Some(item) = self.delegate.get(self.selected_index) {
                    view.title = Some(item.title().to_string().into());
                    view.value = Some(item.value().to_string().into());
                }
                view.open = false;
                view.focus_handle.focus(cx);
            });
        }
    }
}

pub struct Dropdown<D: DropdownDelegate + 'static> {
    id: ElementId,
    focus_handle: FocusHandle,
    picker: View<Picker<DropdownPickerDelegate<D>>>,
    open: bool,
    /// The value of the selected item.
    value: Option<SharedString>,
    title: Option<SharedString>,
}

impl<D> Dropdown<D>
where
    D: DropdownDelegate + 'static,
{
    pub fn new(id: impl Into<ElementId>, delegate: D, cx: &mut ViewContext<Self>) -> Self {
        let picker_delegate = DropdownPickerDelegate {
            delegate,
            dropdown: cx.view().downgrade(),
            selected_index: 0,
        };

        let picker = cx.new_view(|cx| Picker::uniform_list(picker_delegate, cx).no_query());
        Self {
            id: id.into(),
            focus_handle: cx.focus_handle(),
            picker,
            open: false,
            title: None,
            value: None,
        }
    }

    pub fn set_value(&mut self, value: impl Into<SharedString>, cx: &mut ViewContext<Self>) {
        self.value = Some(value.into());
        cx.notify();
    }

    fn up(&mut self, _: &Up, cx: &mut ViewContext<Self>) {
        if !self.open {
            return;
        }
        self.picker.focus_handle(cx).focus(cx);
        cx.dispatch_action(Box::new(picker::SelectPrev));
    }

    fn down(&mut self, _: &Down, cx: &mut ViewContext<Self>) {
        if !self.open {
            self.open = true;
        }

        self.picker.focus_handle(cx).focus(cx);
        cx.dispatch_action(Box::new(picker::SelectNext));
    }

    fn enter(&mut self, _: &Enter, cx: &mut ViewContext<Self>) {
        if !self.open {
            self.open = true;
            cx.notify();
        } else {
            self.picker.focus_handle(cx).focus(cx);
            cx.dispatch_action(Box::new(picker::Confirm));
        }
    }

    fn escape(&mut self, _: &Escape, cx: &mut ViewContext<Self>) {
        self.open = false;
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
            .track_focus(&self.picker.focus_handle(cx))
            .child(self.picker.clone())
            .on_mouse_down_out(|_, cx| {
                cx.dispatch_action(Box::new(Escape));
            })
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
        let group_id = format!("dropdown-group:{}", self.id);
        let title = self.title.clone().unwrap_or_else(|| "Select...".into());
        let focused = self.focus_handle.is_focused(cx);

        div()
            .key_context("Dropdown")
            .group(group_id.clone())
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
                    .px_3()
                    .py_2()
                    .when(focused, |this| this.border_color(cx.theme().ring))
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
                            .child(
                                Icon::new(IconName::ChevronsUpDown)
                                    .size_4()
                                    .text_color(cx.theme().muted_foreground),
                            ),
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

struct DropdownMenuElementState {
    menu_element: Option<AnyElement>,
    layout_id: Option<LayoutId>,
}

impl Default for DropdownMenuElementState {
    fn default() -> Self {
        Self {
            menu_element: None,
            layout_id: None,
        }
    }
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
                        layout_id.clone(),
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
