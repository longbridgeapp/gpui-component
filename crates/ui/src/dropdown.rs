use std::rc::Rc;

use gpui::{
    div, prelude::FluentBuilder as _, Div, ElementId, InteractiveElement, IntoElement,
    ParentElement as _, Render, RenderOnce, SharedString, Stateful, Styled as _, View, ViewContext,
    VisualContext as _, WeakView,
};

use crate::{
    list::ListItem,
    picker::{Picker, PickerDelegate},
    IconName,
};

/// A trait for items that can be displayed in a dropdown.
pub trait DropdownItem {
    fn title(&self) -> &str;
    fn value(&self) -> &str;
}

pub trait DropdownDelegate {
    type Item: DropdownItem;

    fn len(&self) -> usize;
    fn get(&self, index: usize) -> Option<&Self::Item>;
}

struct DropdownPickerDelegate<D: DropdownDelegate> {
    dropdown: WeakView<Dropdown<D>>,
    selected_index: usize,
}

impl<D> PickerDelegate for DropdownPickerDelegate<D: PickerDelegate> {
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
                .child(item.title());
            Some(list_item)
        } else {
            None
        }
    }

    fn dismissed(&mut self, cx: &mut ViewContext<Picker<Self>>) {
        if let Some(view) = self.dropdown.upgrade() {
            cx.update_view(&view, |view, cx| {
                view.open = false;
                cx.notify();
            });
        }
    }

    fn confirm(&mut self, _secondary: bool, cx: &mut ViewContext<Picker<Self>>) {
        if let Some(view) = self.dropdown.upgrade() {
            cx.update_view(&view, |view, cx| {
                if let Some(item) = self.delegate.get(self.selected_index) {
                    view.value = Some(item.value());
                }
                view.open = false;
                cx.notify();
            });
        }
    }
}

pub struct Dropdown<D: DropdownDelegate> {
    base: Stateful<Div>,
    delegate: D,
    picker: View<Picker<DropdownPickerDelegate>>,
    open: bool,
    /// The value of the selected item.
    value: Option<SharedString>,
}

impl<D: DropdownDelegate> Dropdown<D> {
    pub fn new(id: impl Into<ElementId>, delegate: D, cx: &mut ViewContext<Self>) -> Self {
        let delegate = DropdownPickerDelegate {
            // delegate,
            dropdown: cx.view().downgrade(),
            selected_index: 0,
        };

        let picker = cx.new_view(|cx| Picker::uniform_list(delegate, cx));
        Self {
            delegate,
            base: div().id(id.into()),
            picker,
            open: false,
            value: None,
        }
    }
}

impl RenderOnce for Dropdown {
    fn render(self, cx: &mut gpui::WindowContext) -> impl IntoElement {
        self.base
            .child(self.value.unwrap_or_else(|| "Select...".into()))
            .when(self.open, |this| this.child(self.picker))
    }
}
