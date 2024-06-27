use std::rc::Rc;

use gpui::{
    div, prelude::FluentBuilder as _, px, AppContext, Div, ElementId, FocusHandle, FocusableView,
    InteractiveElement, IntoElement, ParentElement as _, Render, RenderOnce, SharedString,
    Stateful, StatefulInteractiveElement as _, Styled as _, View, ViewContext, VisualContext as _,
    WeakView,
};

use crate::{
    h_flex,
    list::ListItem,
    picker::{Picker, PickerDelegate},
    theme::ActiveTheme,
    v_flex, IconName, StyledExt,
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

    fn render_match(
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
            cx.update_view(&view, |view, cx| {
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

        let picker = cx.new_view(|cx| Picker::uniform_list(picker_delegate, cx));
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
}

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
        let is_focused = self.focus_handle.is_focused(cx);

        div()
            .key_context("Dropdown")
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
                    .rounded_sm()
                    .shadow_sm()
                    .px_3()
                    .py_2()
                    .when(is_focused, |this| this.border_color(cx.theme().ring))
                    .on_click(cx.listener(|this, _, cx| {
                        this.open = !this.open;
                        cx.notify();
                    }))
                    .child(
                        v_flex()
                            .items_center()
                            .justify_between()
                            .child(title)
                            .child(IconName::ChevronDown),
                    ),
            )
            .when(self.open, |this| {
                this.child(
                    div()
                        .absolute()
                        // Top is the dropdown input height + border
                        .top(px(50.))
                        .left_0()
                        .bg(cx.theme().background)
                        .border_1()
                        .border_color(cx.theme().input)
                        .rounded_sm()
                        .shadow_md()
                        .track_focus(&self.picker.focus_handle(cx))
                        .child(self.picker.clone()),
                )
            })
    }
}
