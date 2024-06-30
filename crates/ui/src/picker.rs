use std::time::Duration;

use anyhow::Result;
use gpui::{
    actions, div, list, prelude::FluentBuilder as _, px, rems, uniform_list, AppContext,
    ClickEvent, DismissEvent, Div, Element, EventEmitter, FocusHandle, FocusableView,
    InteractiveElement, IntoElement, KeyBinding, Length, ListSizingBehavior, ListState,
    MouseButton, MouseUpEvent, ParentElement as _, Render, SharedString,
    StatefulInteractiveElement as _, Styled as _, Task, UniformListScrollHandle, View, ViewContext,
    VisualContext as _, WindowContext,
};

actions!(
    picker,
    [
        Cancel,
        Confirm,
        SecondaryConfirm,
        SelectPrev,
        SelectNext,
        SelectFirst,
        SelectLast,
    ]
);

pub fn init(cx: &mut AppContext) {
    let context = Some("Picker");
    cx.bind_keys([
        KeyBinding::new("enter", Confirm, context),
        KeyBinding::new("escape", Cancel, context),
        KeyBinding::new("up", SelectPrev, context),
        KeyBinding::new("down", SelectNext, context),
    ]);
}

use crate::{
    divider::Divider,
    empty::Empty,
    input::{TextEvent, TextInput},
    scrollbar::Scrollbar,
    stock::*,
    theme::ActiveTheme,
    StyledExt as _,
};

enum ElementContainer {
    List(ListState),
    UniformList(UniformListScrollHandle),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
enum ContainerKind {
    List,
    UniformList,
}

pub trait PickerDelegate: Sized + 'static {
    type ListItem: IntoElement;

    fn match_count(&self) -> usize;

    /// Return the index of the selected item.
    fn selected_index(&self) -> usize;

    /// Update the selected index.
    fn set_selected_index(&mut self, ix: usize, cx: &mut ViewContext<Picker<Self>>);

    fn selected_index_changed(
        &self,
        _ix: usize,
        _cx: &mut ViewContext<Picker<Self>>,
    ) -> Option<Box<dyn Fn(&mut WindowContext) + 'static>> {
        None
    }

    /// Callback when the picker is confirmed.
    fn confirm(&mut self, _secondary: bool, _cx: &mut ViewContext<Picker<Self>>) {}

    /// Callback when the picker is dismissed.
    fn dismissed(&mut self, _cx: &mut ViewContext<Picker<Self>>) {}

    /// Determine if the picker should be dismissed, return true by default. Return false will abort the dismiss action.
    fn should_dismiss(&self) -> bool {
        true
    }

    /// Override this method to customize the query input header container.
    fn render_query(&self, input: &View<TextInput>, _cx: &mut ViewContext<Picker<Self>>) -> Div {
        v_flex()
            .child(
                h_flex()
                    .overflow_hidden()
                    .flex_none()
                    .h_9()
                    .px_4()
                    .child(input.clone()),
            )
            .child(Divider::horizontal())
    }

    /// Render the list item at the given index.
    fn render_item(
        &self,
        ix: usize,
        selected: bool,
        cx: &mut ViewContext<Picker<Self>>,
    ) -> Option<Self::ListItem>;

    fn separators_after_indices(&self) -> Vec<usize> {
        Vec::new()
    }

    fn update_matches(&mut self, _query: &str, _cx: &mut ViewContext<Picker<Self>>) -> Task<()> {
        Task::ready(())
    }

    fn confirm_update_query(
        &mut self,
        _cx: &mut ViewContext<Picker<Self>>,
    ) -> Option<SharedString> {
        None
    }

    fn selected_as_query(&self) -> Option<SharedString> {
        None
    }
}

impl<D: PickerDelegate> FocusableView for Picker<D> {
    fn focus_handle(&self, cx: &AppContext) -> FocusHandle {
        if let Some(input) = &self.query_input {
            input.focus_handle(cx)
        } else {
            self.head.focus_handle(cx)
        }
    }
}

impl<D: PickerDelegate> EventEmitter<DismissEvent> for Picker<D> {}

struct PendingUpdateMatches {
    delegate_update_matches: Option<Task<()>>,
    _task: Task<Result<()>>,
}

pub struct Picker<D: PickerDelegate> {
    delegate: D,
    element_container: ElementContainer,
    query_input: Option<View<TextInput>>,
    width: Option<Length>,
    max_height: Option<Length>,
    is_modal: bool,
    /// Just a empty view for holding the focus
    head: View<Empty>,
    pending_update_matches: Option<PendingUpdateMatches>,
    show_scrollbar: bool,
    hide_scrollbar_task: Option<Task<()>>,
}

impl<D: PickerDelegate> Picker<D> {
    fn new(
        delegate: D,
        kind: ContainerKind,
        query_input: Option<View<TextInput>>,
        cx: &mut ViewContext<Self>,
    ) -> Self {
        let element_container = match kind {
            ContainerKind::List => {
                let view = cx.view().downgrade();
                ElementContainer::List(ListState::new(
                    0,
                    gpui::ListAlignment::Top,
                    px(1000.),
                    move |ix, cx| {
                        view.upgrade()
                            .map(|view| {
                                view.update(cx, |this, cx| {
                                    this.render_element(cx, ix).into_any_element()
                                })
                            })
                            .unwrap_or_else(|| div().into_any_element())
                    },
                ))
            }
            ContainerKind::UniformList => {
                ElementContainer::UniformList(UniformListScrollHandle::new())
            }
        };

        Self {
            delegate,
            query_input,
            head: cx.new_view(Empty::new),
            width: None,
            is_modal: false,
            max_height: None,
            element_container,
            pending_update_matches: None,
            show_scrollbar: false,
            hide_scrollbar_task: None,
        }
    }

    fn new_query_input(
        placehoder: impl Into<SharedString>,
        cx: &mut ViewContext<Self>,
    ) -> View<TextInput> {
        let input = cx.new_view(|cx| {
            let mut input = TextInput::new(cx).appearance(false);
            input.set_placeholder(placehoder, cx);
            input
        });
        cx.subscribe(&input, Self::on_query_input_event).detach();
        input
    }

    pub fn delegate(&self) -> &D {
        &self.delegate
    }

    pub fn delegate_mut(&mut self) -> &mut D {
        &mut self.delegate
    }

    pub fn list(delegate: D, cx: &mut ViewContext<Self>) -> Self {
        let query_input = Self::new_query_input("Search...", cx);
        Self::new(delegate, ContainerKind::List, Some(query_input), cx)
    }

    pub fn uniform_list(delegate: D, cx: &mut ViewContext<Self>) -> Self {
        let query_input = Self::new_query_input("Search...", cx);
        Self::new(delegate, ContainerKind::UniformList, Some(query_input), cx)
    }

    pub fn width(mut self, width: impl Into<gpui::Length>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn max_height(mut self, max_height: Option<gpui::Length>) -> Self {
        self.max_height = max_height;
        self
    }

    pub fn modal(mut self, modal: bool) -> Self {
        self.is_modal = modal;
        self
    }

    pub fn focus(&self, cx: &mut WindowContext) {
        self.focus_handle(cx).focus(cx);
    }

    /// Hide the query input.
    pub fn no_query(mut self) -> Self {
        self.query_input = None;
        self
    }

    pub fn set_query(&mut self, query: &str, cx: &mut ViewContext<Self>) {
        if let Some(input) = &self.query_input {
            input.update(cx, |this, cx| this.set_text(query.to_string(), cx));
        }
    }

    /// Return the query input string.
    pub fn query(&self, cx: &AppContext) -> SharedString {
        if let Some(input) = &self.query_input {
            input.read(cx).text()
        } else {
            "".into()
        }
    }

    fn render_element_container(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let sizing_behavior = if self.max_height.is_some() {
            ListSizingBehavior::Infer
        } else {
            ListSizingBehavior::Auto
        };

        match &self.element_container {
            ElementContainer::UniformList(scroll_handle) => uniform_list(
                cx.view().clone(),
                "candidates",
                self.delegate.match_count(),
                move |picker, visible_range, cx| {
                    visible_range
                        .map(|ix| picker.render_element(cx, ix))
                        .collect()
                },
            )
            .with_sizing_behavior(sizing_behavior)
            .flex_grow()
            .track_scroll(scroll_handle.clone())
            .into_any_element(),
            ElementContainer::List(state) => list(state.clone())
                .with_sizing_behavior(sizing_behavior)
                .flex_grow()
                .into_any_element(),
        }
    }

    fn render_element(&self, cx: &mut ViewContext<Self>, ix: usize) -> impl IntoElement {
        div()
            .id(("item", ix))
            .cursor_pointer()
            .on_click(cx.listener(move |this, event: &ClickEvent, cx| {
                this.on_click(ix, event.down.modifiers.secondary(), cx)
            }))
            // As of this writing, GPUI intercepts `ctrl-[mouse-event]`s on macOS
            // and produces right mouse button events. This matches platforms norms
            // but means that UIs which depend on holding ctrl down (such as the tab
            // switcher) can't be clicked on. Hence, this handler.
            .on_mouse_up(
                MouseButton::Right,
                cx.listener(move |this, event: &MouseUpEvent, cx| {
                    // We specficially want to use the platform key here, as
                    // ctrl will already be held down for the tab switcher.
                    this.on_click(ix, event.modifiers.platform, cx)
                }),
            )
            .children(
                self.delegate
                    .render_item(ix, ix == self.delegate.selected_index(), cx),
            )
            .when(
                self.delegate.separators_after_indices().contains(&ix),
                |picker| {
                    picker
                        .border_color(cx.theme().border)
                        .border_b_1()
                        .pb(px(-1.0))
                },
            )
    }

    fn render_scrollbar(&self, cx: &mut ViewContext<Self>) -> Option<impl IntoElement> {
        if !self.show_scrollbar {
            return None;
        }

        if let Some(scroll_handle) = match &self.element_container {
            ElementContainer::List(_state) => None,
            ElementContainer::UniformList(scroll_handle) => Some(scroll_handle.clone()),
        } {
            Scrollbar::new(
                cx.view().clone().into(),
                scroll_handle,
                self.delegate.match_count(),
                true,
            )
        } else {
            None
        }
    }

    fn hide_scrollbar(&mut self, cx: &mut ViewContext<Self>) {
        const SCROLLBAR_SHOW_INTERVAL: Duration = Duration::from_secs(1);
        self.show_scrollbar = false;
        self.hide_scrollbar_task = Some(cx.spawn(|panel, mut cx| async move {
            cx.background_executor()
                .timer(SCROLLBAR_SHOW_INTERVAL)
                .await;
            panel
                .update(&mut cx, |panel, cx| {
                    panel.show_scrollbar = false;
                    cx.notify();
                })
                .ok();
        }))
    }

    fn on_click(&mut self, ix: usize, secondary: bool, cx: &mut ViewContext<Self>) {
        cx.stop_propagation();
        cx.prevent_default();
        self.set_selected_index(ix, false, cx);
        self.do_confirm(secondary, cx)
    }

    fn do_confirm(&mut self, secondary: bool, cx: &mut ViewContext<Self>) {
        if let Some(update_query) = self.delegate.confirm_update_query(cx) {
            self.set_query(&update_query, cx);
            self.delegate.set_selected_index(0, cx);
        } else {
            self.delegate.confirm(secondary, cx)
        }
    }

    pub fn set_selected_index(
        &mut self,
        ix: usize,
        scroll_to_index: bool,
        cx: &mut ViewContext<Self>,
    ) {
        let previous_index = self.delegate.selected_index();
        self.delegate.set_selected_index(ix, cx);
        let current_index = self.delegate.selected_index();

        if previous_index != current_index {
            if let Some(action) = self.delegate.selected_index_changed(ix, cx) {
                action(cx);
            }
            if scroll_to_index {
                self.scroll_to_item_index(ix);
            }
        }
    }

    fn scroll_to_item_index(&mut self, ix: usize) {
        match &mut self.element_container {
            ElementContainer::List(state) => state.scroll_to_reveal_item(ix),
            ElementContainer::UniformList(scroll_handle) => scroll_handle.scroll_to_item(ix),
        }
    }

    fn confirm(&mut self, _: &Confirm, cx: &mut ViewContext<Self>) {
        self.do_confirm(false, cx);
    }

    pub fn select_next(&mut self, _: &SelectNext, cx: &mut ViewContext<Self>) {
        let count = self.delegate.match_count();
        if count > 0 {
            let index = self.delegate.selected_index();
            let ix = if index == count - 1 { 0 } else { index + 1 };
            self.set_selected_index(ix, true, cx);
            cx.notify();
        }
    }

    fn select_prev(&mut self, _: &SelectPrev, cx: &mut ViewContext<Self>) {
        let count = self.delegate.match_count();
        if count > 0 {
            let index = self.delegate.selected_index();
            let ix = if index == 0 { count - 1 } else { index - 1 };
            self.set_selected_index(ix, true, cx);
            cx.notify();
        }
    }

    fn select_first(&mut self, _: &SelectFirst, cx: &mut ViewContext<Self>) {
        let count = self.delegate.match_count();
        if count > 0 {
            self.set_selected_index(0, true, cx);
            cx.notify();
        }
    }

    fn select_last(&mut self, _: &SelectLast, cx: &mut ViewContext<Self>) {
        let count = self.delegate.match_count();
        if count > 0 {
            self.delegate.set_selected_index(count - 1, cx);
            self.set_selected_index(count - 1, true, cx);
            cx.notify();
        }
    }

    pub fn cancel(&mut self, _: &Cancel, cx: &mut ViewContext<Self>) {
        if self.delegate.should_dismiss() {
            self.delegate.dismissed(cx);
            cx.emit(DismissEvent);
        }
    }

    pub fn refresh(&mut self, cx: &mut ViewContext<Self>) {
        let query = self.query(cx);
        self.update_matches(&query, cx);
    }

    pub fn update_matches(&mut self, query: &str, cx: &mut ViewContext<Self>) {
        let delegate_pending_update_matches = self.delegate.update_matches(query, cx);

        self.matches_updated(cx);
        // This struct ensures that we can synchronously drop the task returned by the
        // delegate's `update_matches` method and the task that the picker is spawning.
        // If we simply capture the delegate's task into the picker's task, when the picker's
        // task gets synchronously dropped, the delegate's task would keep running until
        // the picker's task has a chance of being scheduled, because dropping a task happens
        // asynchronously.
        self.pending_update_matches = Some(PendingUpdateMatches {
            delegate_update_matches: Some(delegate_pending_update_matches),
            _task: cx.spawn(|this, mut cx| async move {
                let delegate_pending_update_matches = this.update(&mut cx, |this, _| {
                    this.pending_update_matches
                        .as_mut()
                        .unwrap()
                        .delegate_update_matches
                        .take()
                        .unwrap()
                })?;
                delegate_pending_update_matches.await;
                this.update(&mut cx, |this, cx| {
                    this.matches_updated(cx);
                })
            }),
        });
    }

    fn matches_updated(&mut self, cx: &mut ViewContext<Self>) {
        if let ElementContainer::List(state) = &mut self.element_container {
            state.reset(self.delegate.match_count());
        }

        let index = self.delegate.selected_index();
        self.scroll_to_item_index(index);
        // self.pending_update_matches = None;
        // if let Some(secondary) = self.confirm_on_update.take() {
        //     self.confirm(secondary, cx);
        // }
        cx.notify();
    }

    fn on_query_input_event(
        &mut self,
        _: View<TextInput>,
        event: &TextEvent,
        cx: &mut ViewContext<Self>,
    ) {
        #[allow(clippy::single_match)]
        match event {
            TextEvent::Input { text } => {
                self.set_query(text, cx);
                self.refresh(cx);
            }
        }
    }
}

impl<D: PickerDelegate> Render for Picker<D> {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle(cx);

        v_flex()
            .id("picker")
            .key_context("Picker")
            .size_full()
            .track_focus(&focus_handle)
            .when_some(self.width, |el, width| el.w(width))
            .overflow_hidden()
            .when(self.is_modal, |this| this.elevation_3(cx))
            .on_action(cx.listener(Self::select_next))
            .on_action(cx.listener(Self::select_prev))
            .on_action(cx.listener(Self::select_first))
            .on_action(cx.listener(Self::select_last))
            .on_action(cx.listener(Self::cancel))
            .on_action(cx.listener(Self::confirm))
            .on_hover(cx.listener(|this, hovered: &bool, cx| {
                if *hovered {
                    this.show_scrollbar = true;
                    this.hide_scrollbar_task.take();
                    cx.notify();
                } else {
                    this.hide_scrollbar(cx);
                }
            }))
            // Render Query Input header
            .child(match self.query_input {
                Some(ref input) => self.delegate.render_query(input, cx),
                None => div().child(self.head.clone()),
            })
            .when(self.delegate.match_count() > 0, |this| {
                this.child(
                    v_flex()
                        .flex_grow()
                        .min_h(px(100.))
                        .when_some(self.max_height, |div, max_h| div.max_h(max_h))
                        .overflow_hidden()
                        .child(self.render_element_container(cx)), // .children(self.render_scrollbar(cx)),
                )
            })
            .when(self.delegate.match_count() == 0, |el| {
                el.child(
                    v_flex()
                        .h_full()
                        .size_full()
                        .h_16()
                        .items_center()
                        .content_center()
                        .justify_center()
                        .text_color(cx.theme().muted_foreground)
                        .child("No matched."),
                )
            })
            .on_mouse_down_out(cx.listener(|_, _, cx| {
                cx.dispatch_action(Box::new(Cancel));
            }))
    }
}
