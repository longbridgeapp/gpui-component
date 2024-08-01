use std::{
    any::Any,
    cmp,
    collections::{BTreeSet, HashMap},
    fmt, mem,
    ops::ControlFlow,
    rc::Rc,
    sync::Arc,
};

use anyhow::Result;
use gpui::{
    actions, div, impl_actions, prelude::FluentBuilder as _, px, AppContext, DefiniteLength,
    DragMoveEvent, Element as _, EntityId, EventEmitter, FocusHandle, FocusOutEvent, FocusableView,
    InteractiveElement as _, IntoElement, KeyContext, ParentElement, Pixels, Point, Render,
    ScrollHandle, StatefulInteractiveElement, Styled, Subscription, Task, View, ViewContext,
    VisualContext as _, WeakFocusHandle, WeakView, WindowContext,
};
use serde::Deserialize;

use crate::util::ResultExt;
use ui::{
    label::Label,
    tab::{Tab, TabBar},
    theme::{ActiveTheme, Colorize as _},
    tooltip::Tooltip,
    v_flex, Icon, IconName, Selectable, StyledExt,
};

use super::{
    item::{ItemHandle, TabContentParams},
    pane_group::SplitDirection,
    workspace::Workspace,
};

#[derive(Clone, Deserialize, PartialEq, Debug)]
pub struct ActivateItem(pub usize);

impl_actions!(pane, [ActivateItem]);

actions!(
    pane,
    [
        CloseActiveItem,
        CloseInactiveItems,
        CloseAllItems,
        ActivatePrevItem,
        ActivateNextItem,
        ActivateLastItem,
        CloseItemsToTheLeft,
        CloseItemsToTheRight,
        GoBack,
        GoForward,
        ReopenClosedItem,
        SplitLeft,
        SplitUp,
        SplitRight,
        SplitDown,
    ]
);

pub enum Event {
    AddItem { item: Box<dyn ItemHandle> },
    ActivateItem { local: bool },
    Remove,
    RemoveItem { item_id: EntityId },
    Split(SplitDirection),
    ChangeItemTitle,
    Focus,
    ZoomIn,
    ZoomOut,
}

impl fmt::Debug for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::AddItem { item } => f
                .debug_struct("AddItem")
                .field("item", &item.item_id())
                .finish(),
            Event::ActivateItem { local } => f
                .debug_struct("ActivateItem")
                .field("local", local)
                .finish(),
            Event::Remove => f.write_str("Remove"),
            Event::RemoveItem { item_id } => f
                .debug_struct("RemoveItem")
                .field("item_id", item_id)
                .finish(),
            Event::Split(direction) => f
                .debug_struct("Split")
                .field("direction", direction)
                .finish(),
            Event::ChangeItemTitle => f.write_str("ChangeItemTitle"),
            Event::Focus => f.write_str("Focus"),
            Event::ZoomIn => f.write_str("ZoomIn"),
            Event::ZoomOut => f.write_str("ZoomOut"),
        }
    }
}

#[derive(Clone)]
pub struct DraggedTab {
    pub pane: View<Pane>,
    pub item: Box<dyn ItemHandle>,
    pub ix: usize,
    pub detail: usize,
    pub is_active: bool,
}

impl Render for DraggedTab {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let label = self.item.tab_content(
            TabContentParams {
                detail: Some(self.detail),
                selected: false,
            },
            cx,
        );
        Tab::new("", label).selected(self.is_active).into_element()
    }
}

/// A pane
pub struct Pane {
    pub(crate) workspace: WeakView<Workspace>,
    focus_handle: FocusHandle,
    items: Vec<Box<dyn ItemHandle>>,
    zoomed: bool,
    was_focused: bool,
    last_focus_handle_by_item: HashMap<EntityId, WeakFocusHandle>,
    active_item_index: usize,
    drag_split_direction: Option<SplitDirection>,
    can_drop_predicate: Option<Arc<dyn Fn(&dyn Any, &mut WindowContext) -> bool>>,
    custom_drop_handle:
        Option<Arc<dyn Fn(&mut Pane, &dyn Any, &mut ViewContext<Pane>) -> ControlFlow<(), ()>>>,
    can_split: bool,
    should_display_tab_bar: Rc<dyn Fn(&ViewContext<Pane>) -> bool>,
    tab_bar_scroll_handle: ScrollHandle,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<Event> for Pane {}

impl Pane {
    pub fn new(
        workspace: WeakView<Workspace>,
        can_drop_predicate: Option<Arc<dyn Fn(&dyn Any, &mut WindowContext) -> bool + 'static>>,
        cx: &mut ViewContext<Self>,
    ) -> Self {
        let focus_handle = cx.focus_handle();

        let subscriptions = vec![
            cx.on_focus(&focus_handle, Pane::focus_in),
            cx.on_focus_in(&focus_handle, Pane::focus_in),
            cx.on_focus_out(&focus_handle, Pane::focus_out),
        ];

        let _handle = cx.view().downgrade();
        Self {
            workspace,
            focus_handle,
            items: Vec::new(),
            zoomed: false,
            was_focused: false,
            active_item_index: 0,
            drag_split_direction: None,
            last_focus_handle_by_item: HashMap::new(),
            can_drop_predicate,
            custom_drop_handle: None,
            can_split: true,
            should_display_tab_bar: Rc::new(|_| true),
            tab_bar_scroll_handle: ScrollHandle::new(),
            _subscriptions: subscriptions,
        }
    }

    pub fn set_can_split(&mut self, can_split: bool, cx: &mut ViewContext<Self>) {
        self.can_split = can_split;
        cx.notify();
    }

    pub fn set_should_display_tab_bar<F>(&mut self, f: F)
    where
        F: 'static + Fn(&ViewContext<Pane>) -> bool,
    {
        self.should_display_tab_bar = Rc::new(f);
    }

    pub fn set_custom_drop_handle<F>(&mut self, cx: &mut ViewContext<Self>, handle: F)
    where
        F: 'static + Fn(&mut Pane, &dyn Any, &mut ViewContext<Pane>) -> ControlFlow<(), ()>,
    {
        self.custom_drop_handle = Some(Arc::new(handle));
        cx.notify();
    }

    pub fn close_item_by_id(
        &mut self,
        item_id_to_close: EntityId,
        cx: &mut ViewContext<Self>,
    ) -> Task<Result<()>> {
        self.close_items(cx, move |view_id| view_id == item_id_to_close)
    }

    pub fn close_items(
        &mut self,
        cx: &mut ViewContext<Pane>,
        should_close: impl Fn(EntityId) -> bool,
    ) -> Task<Result<()>> {
        let mut items_to_close = Vec::new();

        for item in &self.items {
            if should_close(item.item_id()) {
                items_to_close.push(item.boxed_clone());
            }
        }

        // let workspace = self.workspace.clone();
        cx.spawn(|pane, mut cx| async move {
            for item in items_to_close {
                // Remove the item from the pane.
                pane.update(&mut cx, |pane, cx| {
                    if let Some(item_ix) = pane
                        .items
                        .iter()
                        .position(|i| i.item_id() == item.item_id())
                    {
                        pane.remove_item(item_ix, false, cx);
                    }
                })
                .ok();
            }

            pane.update(&mut cx, |_, cx| cx.notify()).ok();
            Ok(())
        })
    }

    pub fn remove_item(
        &mut self,
        item_index: usize,
        activate_pane: bool,
        cx: &mut ViewContext<Self>,
    ) {
        let close_pane_if_empty = true;
        // self.activation_history
        //     .retain(|entry| entry.entity_id != self.items[item_index].item_id());

        if item_index == self.active_item_index {
            let index_to_activate = item_index.min(self.items.len()).saturating_sub(1);

            let should_activate = activate_pane || self.has_focus(cx);
            if self.items.len() == 1 && should_activate {
                self.focus_handle.focus(cx);
            } else {
                self.activate_item(index_to_activate, should_activate, should_activate, cx);
            }
        }

        let item = self.items.remove(item_index);
        cx.emit(Event::RemoveItem {
            item_id: item.item_id(),
        });
        if self.items.is_empty() {
            item.deactivated(cx);
            if close_pane_if_empty {
                cx.emit(Event::Remove);
            }
        }

        if item_index < self.active_item_index {
            self.active_item_index -= 1;
        }

        // let mode = self.nav_history.mode();
        // self.nav_history.set_mode(NavigationMode::ClosingItem);
        // item.deactivated(cx);
        // self.nav_history.set_mode(mode);

        if self.items.is_empty() && close_pane_if_empty && self.zoomed {
            cx.emit(Event::ZoomOut);
        }

        cx.notify();
    }

    pub fn add_item(
        &mut self,
        item: Box<dyn ItemHandle>,
        activate_pane: bool,
        focus_item: bool,
        destination_index: Option<usize>,
        cx: &mut ViewContext<Self>,
    ) {
        // If no destination index is specified, add or move the item after the active item.
        let mut insertion_index = {
            cmp::min(
                if let Some(destination_index) = destination_index {
                    destination_index
                } else {
                    self.active_item_index + 1
                },
                self.items.len(),
            )
        };

        let existing_item_index = self
            .items
            .iter()
            .position(|existing_item| existing_item.item_id() == item.item_id());

        if let Some(existing_item_index) = existing_item_index {
            // If the item already exists, move it to the desired destination and activate it

            if existing_item_index != insertion_index {
                let existing_item_is_active = existing_item_index == self.active_item_index;

                // If the caller didn't specify a destination and the added item is already
                // the active one, don't move it
                if existing_item_is_active && destination_index.is_none() {
                    insertion_index = existing_item_index;
                } else {
                    self.items.remove(existing_item_index);
                    if existing_item_index < self.active_item_index {
                        self.active_item_index -= 1;
                    }
                    insertion_index = insertion_index.min(self.items.len());

                    self.items.insert(insertion_index, item.clone());

                    if existing_item_is_active {
                        self.active_item_index = insertion_index;
                    } else if insertion_index <= self.active_item_index {
                        self.active_item_index += 1;
                    }
                }

                cx.notify();
            }

            self.activate_item(insertion_index, activate_pane, focus_item, cx);
        } else {
            self.items.insert(insertion_index, item.clone());

            if insertion_index <= self.active_item_index {
                self.active_item_index += 1;
            }

            self.activate_item(insertion_index, activate_pane, focus_item, cx);
            cx.notify();
        }

        cx.emit(Event::AddItem { item });
    }

    pub fn items_len(&self) -> usize {
        self.items.len()
    }

    pub fn items(&self) -> impl DoubleEndedIterator<Item = &Box<dyn ItemHandle>> {
        self.items.iter()
    }

    pub fn active_item(&self) -> Option<Box<dyn ItemHandle>> {
        self.items.get(self.active_item_index).cloned()
    }

    pub fn pixel_position_of_cursor(&self, cx: &AppContext) -> Option<Point<Pixels>> {
        self.items
            .get(self.active_item_index)?
            .pixel_position_of_cursor(cx)
    }

    pub fn activate_item(
        &mut self,
        index: usize,
        activate_pane: bool,
        focus_item: bool,
        cx: &mut ViewContext<Self>,
    ) {
        if index < self.items.len() {
            let prev_active_item_ix = mem::replace(&mut self.active_item_index, index);
            if prev_active_item_ix != self.active_item_index {
                if let Some(prev_item) = self.items.get(prev_active_item_ix) {
                    prev_item.deactivated(cx);
                }
            }

            cx.emit(Event::ActivateItem {
                local: activate_pane,
            });

            if focus_item {
                self.focus_active_item(cx);
            }

            self.tab_bar_scroll_handle.scroll_to_item(index);
            cx.notify();
        }
    }

    pub fn activate_prev_item(&mut self, activate_pane: bool, cx: &mut ViewContext<Self>) {
        let mut index = self.active_item_index;
        if index > 0 {
            index -= 1;
        } else if !self.items.is_empty() {
            index = self.items.len() - 1;
        }
        self.activate_item(index, activate_pane, activate_pane, cx);
    }

    pub fn activate_next_item(&mut self, activate_pane: bool, cx: &mut ViewContext<Self>) {
        let mut index = self.active_item_index;
        if index + 1 < self.items.len() {
            index += 1;
        } else {
            index = 0;
        }
        self.activate_item(index, activate_pane, activate_pane, cx);
    }

    pub fn close_active_item(
        &mut self,
        _: &CloseActiveItem,
        cx: &mut ViewContext<Self>,
    ) -> Option<Task<Result<()>>> {
        if self.items.is_empty() {
            // cx.dispatch_action(Box::new(CloseWindow));
            return None;
        }

        let active_item_id = self.items[self.active_item_index].item_id();
        Some(self.close_item_by_id(active_item_id, cx))
    }

    pub fn close_inactive_items(
        &mut self,
        _: &CloseInactiveItems,
        cx: &mut ViewContext<Self>,
    ) -> Option<Task<Result<()>>> {
        if self.items.is_empty() {
            return None;
        }

        let active_item_id = self.items[self.active_item_index].item_id();
        Some(self.close_items(cx, move |item_id| item_id != active_item_id))
    }

    pub fn close_items_to_the_left(
        &mut self,
        _: &CloseItemsToTheLeft,
        cx: &mut ViewContext<Self>,
    ) -> Option<Task<Result<()>>> {
        if self.items.is_empty() {
            return None;
        }
        let active_item_id = self.items[self.active_item_index].item_id();
        Some(self.close_items_to_the_left_by_id(active_item_id, cx))
    }

    pub fn close_items_to_the_left_by_id(
        &mut self,
        item_id: EntityId,
        cx: &mut ViewContext<Self>,
    ) -> Task<Result<()>> {
        let item_ids: Vec<_> = self
            .items()
            .take_while(|item| item.item_id() != item_id)
            .map(|item| item.item_id())
            .collect();
        self.close_items(cx, move |item_id| item_ids.contains(&item_id))
    }

    pub fn close_items_to_the_right(
        &mut self,
        _: &CloseItemsToTheRight,
        cx: &mut ViewContext<Self>,
    ) -> Option<Task<Result<()>>> {
        if self.items.is_empty() {
            return None;
        }
        let active_item_id = self.items[self.active_item_index].item_id();
        Some(self.close_items_to_the_right_by_id(active_item_id, cx))
    }

    pub fn close_items_to_the_right_by_id(
        &mut self,
        item_id: EntityId,
        cx: &mut ViewContext<Self>,
    ) -> Task<Result<()>> {
        let item_ids: Vec<_> = self
            .items()
            .rev()
            .take_while(|item| item.item_id() != item_id)
            .map(|item| item.item_id())
            .collect();
        self.close_items(cx, move |item_id| item_ids.contains(&item_id))
    }

    pub fn close_all_items(
        &mut self,
        _: &CloseAllItems,
        cx: &mut ViewContext<Self>,
    ) -> Option<Task<Result<()>>> {
        if self.items.is_empty() {
            return None;
        }

        Some(self.close_items(cx, |_| true))
    }

    pub fn focus(&mut self, cx: &mut ViewContext<Pane>) {
        cx.focus(&self.focus_handle);
    }

    pub fn focus_active_item(&mut self, cx: &mut ViewContext<Self>) {
        if let Some(active_item) = self.active_item() {
            let focus_handle = active_item.focus_handle(cx);
            cx.focus(&focus_handle);
        }
    }

    pub fn has_focus(&self, cx: &WindowContext) -> bool {
        // We not only check whether our focus handle contains focus, but also
        // whether the active_item might have focus, because we might have just activated an item
        // but that hasn't rendered yet.
        // So before the next render, we might have transferred focus
        // to the item and `focus_handle.contains_focus` returns false because the `active_item`
        // is not hooked up to us in the dispatch tree.
        self.focus_handle.contains_focused(cx)
            || self
                .active_item()
                .map_or(false, |item| item.focus_handle(cx).contains_focused(cx))
    }

    pub fn split(&mut self, direction: SplitDirection, cx: &mut ViewContext<Self>) {
        cx.emit(Event::Split(direction));
    }

    fn handle_drag_move<T>(&mut self, event: &DragMoveEvent<T>, _cx: &mut ViewContext<Self>) {
        if !self.can_split {
            return;
        }

        let rect = event.bounds.size;
        let size = event.bounds.size.width.min(event.bounds.size.height) * 0.33;
        let relative_cursor = Point::new(
            event.event.position.x - event.bounds.left(),
            event.event.position.y - event.bounds.top(),
        );

        let direction = if relative_cursor.x < size
            || relative_cursor.x > rect.width - size
            || relative_cursor.y < size
            || relative_cursor.y > rect.height - size
        {
            [
                SplitDirection::Up,
                SplitDirection::Right,
                SplitDirection::Down,
                SplitDirection::Left,
            ]
            .iter()
            .min_by_key(|side| match side {
                SplitDirection::Up => relative_cursor.y,
                SplitDirection::Right => rect.width - relative_cursor.x,
                SplitDirection::Down => rect.height - relative_cursor.y,
                SplitDirection::Left => relative_cursor.x,
            })
            .cloned()
        } else {
            None
        };

        if direction != self.drag_split_direction {
            self.drag_split_direction = direction;
        }
    }

    fn handle_tab_drop(
        &mut self,
        dragged_tab: &DraggedTab,
        ix: usize,
        cx: &mut ViewContext<'_, Self>,
    ) {
        if let Some(custom_drop_handle) = self.custom_drop_handle.clone() {
            if let ControlFlow::Break(()) = custom_drop_handle(self, dragged_tab, cx) {
                return;
            }
        }
        let mut to_pane = cx.view().clone();
        let split_direction = self.drag_split_direction;
        let item_id = dragged_tab.item.item_id();

        let from_pane = dragged_tab.pane.clone();
        self.workspace
            .update(cx, |_, cx| {
                cx.defer(move |workspace, cx| {
                    if let Some(split_direction) = split_direction {
                        to_pane = workspace.split_pane(to_pane, split_direction, cx);
                    }
                    workspace.move_item(from_pane, to_pane, item_id, ix, cx);
                });
            })
            .log_err();
    }

    fn focus_in(&mut self, cx: &mut ViewContext<Self>) {
        if !self.was_focused {
            self.was_focused = true;
            cx.emit(Event::Focus);
            cx.notify();
        }

        if let Some(active_item) = self.active_item() {
            if self.focus_handle.is_focused(cx) {
                // Pane was focused directly. We need to either focus a view inside the active item,
                // or focus the active item itself
                if let Some(weak_last_focus_handle) =
                    self.last_focus_handle_by_item.get(&active_item.item_id())
                {
                    if let Some(focus_handle) = weak_last_focus_handle.upgrade() {
                        focus_handle.focus(cx);
                        return;
                    }
                }

                active_item.focus_handle(cx).focus(cx);
            } else if let Some(focused) = cx.focused() {
                self.last_focus_handle_by_item
                    .insert(active_item.item_id(), focused.downgrade());
            }
        }
    }

    fn focus_out(&mut self, _event: FocusOutEvent, cx: &mut ViewContext<Self>) {
        self.was_focused = false;
        cx.notify();
    }

    fn render_tab(
        &self,
        ix: usize,
        item: &Box<dyn ItemHandle>,
        detail: usize,
        cx: &mut ViewContext<'_, Pane>,
    ) -> impl IntoElement {
        let is_active = ix == self.active_item_index;

        let label = item.tab_content(
            TabContentParams {
                detail: Some(detail),
                selected: is_active,
            },
            cx,
        );

        let item_id = item.item_id();
        let _is_first_item = ix == 0;
        let _is_last_item = ix == self.items.len() - 1;
        let _position_relative_to_active_item = ix.cmp(&self.active_item_index);

        Tab::new(ix, label)
            .group("tab")
            .px(px(5.))
            .prefix(div().size(px(13.)).into_any_element())
            .gap_1p5()
            .suffix(
                div()
                    .id("close-tab")
                    .p(px(0.))
                    .rounded_sm()
                    .invisible()
                    .child(Icon::new(IconName::Close).size(px(12.)))
                    .hover(|this| this.bg(cx.theme().accent.darken(0.1)))
                    .active(|this| this.bg(cx.theme().accent.darken(0.2)))
                    .on_click(cx.listener(move |pane, _, cx| {
                        pane.close_item_by_id(item_id, cx).detach_and_log_err(cx);
                    }))
                    .group_hover("tab", |this| this.visible())
                    .into_any(),
            )
            .selected(is_active)
            .on_click(
                cx.listener(move |pane: &mut Self, _, cx| pane.activate_item(ix, true, true, cx)),
            )
            .on_drag(
                DraggedTab {
                    item: item.boxed_clone(),
                    pane: cx.view().clone(),
                    detail,
                    is_active,
                    ix,
                },
                |tab, cx| cx.new_view(|_| tab.clone()),
            )
            .drag_over::<DraggedTab>(|tab, _, cx| {
                tab.rounded_l_none()
                    .border_l_2()
                    .border_r_0()
                    .border_color(cx.theme().drag_border)
            })
            .drag_over::<DraggedSelection>(|tab, _, cx| tab.bg(cx.theme().drop_target))
            .when_some(self.can_drop_predicate.clone(), |this, p| {
                this.can_drop(move |a, cx| p(a, cx))
            })
            .on_drop(cx.listener(move |this, dragged_tab: &DraggedTab, cx| {
                this.drag_split_direction = None;
                this.handle_tab_drop(dragged_tab, ix, cx)
            }))
            .when_some(item.tab_tooltip(cx), |tab, text| {
                tab.tooltip(move |cx| Tooltip::new(text.clone(), cx))
            })
    }

    fn render_tab_bar(&mut self, cx: &mut ViewContext<'_, Pane>) -> impl IntoElement {
        //     let navigate_backward = IconButton::new("navigate_backward", IconName::ArrowLeft)
        //     .shape(IconButtonShape::Square)
        //     .icon_size(IconSize::Small)
        //     .on_click({
        //         let view = cx.view().clone();
        //         move |_, cx| view.update(cx, Self::navigate_backward)
        //     })
        //     .disabled(!self.can_navigate_backward())
        //     .tooltip(|cx| Tooltip::for_action("Go Back", &GoBack, cx));

        // let navigate_forward = IconButton::new("navigate_forward", IconName::ArrowRight)
        //     .shape(IconButtonShape::Square)
        //     .icon_size(IconSize::Small)
        //     .on_click({
        //         let view = cx.view().clone();
        //         move |_, cx| view.update(cx, Self::navigate_forward)
        //     })
        //     .disabled(!self.can_navigate_forward())
        //     .tooltip(|cx| Tooltip::for_action("Go Forward", &GoForward, cx));

        TabBar::new("tab-bar")
            .track_scroll(self.tab_bar_scroll_handle.clone())
            .children(
                self.items
                    .iter()
                    .enumerate()
                    .zip(tab_details(&self.items, cx))
                    .map(|((ix, item), detail)| self.render_tab(ix, item, detail, cx)),
            )
            .child(
                div()
                    .id("tab-bar-drop-target")
                    .min_w_6()
                    .child("")
                    .h_full()
                    .flex_grow()
                    .drag_over::<DraggedTab>(|bar, _, cx| bar.bg(cx.theme().drop_target))
                    .drag_over::<DraggedSelection>(|bar, _, cx| bar.bg(cx.theme().drop_target))
                    .on_drop(cx.listener(move |this, dragged_tab: &DraggedTab, cx| {
                        this.drag_split_direction = None;
                        this.handle_tab_drop(dragged_tab, this.items.len(), cx)
                    })),
            )
    }

    pub fn set_zoomed(&mut self, zoomed: bool, cx: &mut ViewContext<Self>) {
        self.zoomed = zoomed;
        cx.notify();
    }

    pub fn is_zoomed(&self) -> bool {
        self.zoomed
    }

    pub fn active_item_index(&self) -> usize {
        self.active_item_index
    }
}

pub fn tab_details(items: &Vec<Box<dyn ItemHandle>>, cx: &AppContext) -> Vec<usize> {
    let mut tab_details = items.iter().map(|_| 0).collect::<Vec<_>>();
    let mut tab_descriptions = HashMap::new();
    let mut done = false;
    while !done {
        done = true;

        // Store item indices by their tab description.
        for (ix, (item, detail)) in items.iter().zip(&tab_details).enumerate() {
            if let Some(description) = item.tab_description(*detail, cx) {
                if *detail == 0
                    || Some(&description) != item.tab_description(detail - 1, cx).as_ref()
                {
                    tab_descriptions
                        .entry(description)
                        .or_insert(Vec::new())
                        .push(ix);
                }
            }
        }

        // If two or more items have the same tab description, increase their level
        // of detail and try again.
        for (_, item_ixs) in tab_descriptions.drain() {
            if item_ixs.len() > 1 {
                done = false;
                for ix in item_ixs {
                    tab_details[ix] += 1;
                }
            }
        }
    }

    tab_details
}

impl FocusableView for Pane {
    fn focus_handle(&self, _cx: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Pane {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let mut key_context = KeyContext::new_with_defaults();
        key_context.add("Pane");
        if self.active_item().is_none() {
            key_context.add("EmptyPane");
        }

        let should_display_tab_bar = self.should_display_tab_bar.clone();
        let display_tab_bar = should_display_tab_bar(cx);

        v_flex()
            .key_context(key_context)
            .track_focus(&self.focus_handle)
            .size_full()
            .flex_none()
            .overflow_hidden()
            .on_action(cx.listener(|pane, _: &SplitLeft, cx| pane.split(SplitDirection::Left, cx)))
            .on_action(cx.listener(|pane, _: &SplitUp, cx| pane.split(SplitDirection::Up, cx)))
            .on_action(
                cx.listener(|pane, _: &SplitRight, cx| pane.split(SplitDirection::Right, cx)),
            )
            .on_action(cx.listener(|pane, _: &SplitDown, cx| pane.split(SplitDirection::Down, cx)))
            // .on_action(cx.listener(|pane, _: &GoBack, cx| pane.navigate_backward(cx)))
            // .on_action(cx.listener(|pane, _: &GoForward, cx| pane.navigate_forward(cx)))
            // .on_action(cx.listener(Pane::toggle_zoom))
            .on_action(cx.listener(|pane: &mut Pane, action: &ActivateItem, cx| {
                pane.activate_item(action.0, true, true, cx);
            }))
            .on_action(cx.listener(|pane: &mut Pane, _: &ActivateLastItem, cx| {
                pane.activate_item(pane.items.len() - 1, true, true, cx);
            }))
            .on_action(cx.listener(|pane: &mut Pane, _: &ActivatePrevItem, cx| {
                pane.activate_prev_item(true, cx);
            }))
            .on_action(cx.listener(|pane: &mut Pane, _: &ActivateNextItem, cx| {
                pane.activate_next_item(true, cx);
            }))
            .on_action(
                cx.listener(|pane: &mut Self, action: &CloseActiveItem, cx| {
                    if let Some(task) = pane.close_active_item(action, cx) {
                        task.detach_and_log_err(cx)
                    }
                }),
            )
            .on_action(
                cx.listener(|pane: &mut Self, action: &CloseInactiveItems, cx| {
                    if let Some(task) = pane.close_inactive_items(action, cx) {
                        task.detach_and_log_err(cx)
                    }
                }),
            )
            .on_action(
                cx.listener(|pane: &mut Self, action: &CloseItemsToTheLeft, cx| {
                    if let Some(task) = pane.close_items_to_the_left(action, cx) {
                        task.detach_and_log_err(cx)
                    }
                }),
            )
            .on_action(
                cx.listener(|pane: &mut Self, action: &CloseItemsToTheRight, cx| {
                    if let Some(task) = pane.close_items_to_the_right(action, cx) {
                        task.detach_and_log_err(cx)
                    }
                }),
            )
            .on_action(cx.listener(|pane: &mut Self, action: &CloseAllItems, cx| {
                if let Some(task) = pane.close_all_items(action, cx) {
                    task.detach_and_log_err(cx)
                }
            }))
            .on_action(
                cx.listener(|pane: &mut Self, action: &CloseActiveItem, cx| {
                    if let Some(task) = pane.close_active_item(action, cx) {
                        task.detach_and_log_err(cx)
                    }
                }),
            )
            .when(self.active_item().is_some() && display_tab_bar, |pane| {
                pane.child(self.render_tab_bar(cx))
            })
            .child({
                // main content
                div()
                    .flex_1()
                    .relative()
                    .group("")
                    .on_drag_move::<DraggedTab>(cx.listener(Self::handle_drag_move))
                    .on_drag_move::<DraggedSelection>(cx.listener(Self::handle_drag_move))
                    .map(|div| {
                        if let Some(item) = self.active_item() {
                            div.v_flex()
                                .id("pane-item-container")
                                .size_full()
                                .overflow_y_scroll()
                                .child(item.to_any())
                        } else {
                            div.id("pane-item-container")
                                .h_flex()
                                .size_full()
                                .justify_center()
                                .child(Label::new("No panel."))
                        }
                    })
                    .child(
                        // drag target
                        div()
                            .invisible()
                            .absolute()
                            .bg(cx.theme().drop_target)
                            .group_drag_over::<DraggedTab>("", |style| style.visible())
                            .group_drag_over::<DraggedSelection>("", |style| style.visible())
                            .when_some(self.can_drop_predicate.clone(), |this, p| {
                                this.can_drop(move |a, cx| p(a, cx))
                            })
                            .on_drop(cx.listener(move |this, dragged_tab, cx| {
                                this.handle_tab_drop(dragged_tab, this.active_item_index(), cx)
                            }))
                            .map(|div| {
                                let size = DefiniteLength::Fraction(0.5);
                                match self.drag_split_direction {
                                    None => div.top_0().right_0().bottom_0().left_0(),
                                    Some(SplitDirection::Up) => {
                                        div.top_0().left_0().right_0().h(size)
                                    }
                                    Some(SplitDirection::Down) => {
                                        div.left_0().bottom_0().right_0().h(size)
                                    }
                                    Some(SplitDirection::Left) => {
                                        div.top_0().left_0().bottom_0().w(size)
                                    }
                                    Some(SplitDirection::Right) => {
                                        div.top_0().bottom_0().right_0().w(size)
                                    }
                                }
                            }),
                    )
            })
        // .on_mouse_down(
        //     MouseButton::Navigate(NavigationDirection::Back),
        //     cx.listener(|pane, _, cx| {
        //         if let Some(workspace) = pane.workspace.upgrade() {
        //             let pane = cx.view().downgrade();
        //             cx.window_context().defer(move |cx| {
        //                 workspace.update(cx, |workspace, cx| {
        //                     workspace.go_back(pane, cx).detach_and_log_err(cx)
        //                 })
        //             })
        //         }
        //     }),
        // )
        // .on_mouse_down(
        //     MouseButton::Navigate(NavigationDirection::Forward),
        //     cx.listener(|pane, _, cx| {
        //         if let Some(workspace) = pane.workspace.upgrade() {
        //             let pane = cx.view().downgrade();
        //             cx.window_context().defer(move |cx| {
        //                 workspace.update(cx, |workspace, cx| {
        //                     workspace.go_forward(pane, cx).detach_and_log_err(cx)
        //                 })
        //             })
        //         }
        //     }),
        // )
    }
}

/// A group of selected entries from project panel.
#[derive(Debug)]
pub struct DraggedSelection {
    pub active_selection: SelectedEntry,
    pub marked_selections: Arc<BTreeSet<SelectedEntry>>,
}

/// A selected entry in e.g. project panel.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SelectedEntry {
    pub entry_id: EntityId,
}
