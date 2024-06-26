use std::time::Duration;

use anyhow::Result;
use gpui::{
    actions, div, list, prelude::FluentBuilder as _, px, rems, uniform_list, AppContext,
    ClickEvent, DismissEvent, Div, EventEmitter, FocusHandle, FocusableView,
    InteractiveElement as _, IntoElement, Length, ListSizingBehavior, ListState, MouseButton,
    MouseUpEvent, ParentElement as _, Render, SharedString, StatefulInteractiveElement as _,
    Styled as _, Task, UniformListScrollHandle, View, ViewContext, VisualContext as _,
    WindowContext,
};

actions!(
    picker,
    [
        UseSelectedQuery,
        Cancel,
        Confirm,
        SecondaryConfirm,
        SelectPrev,
        SelectNext,
        SelectFirst,
        SelectLast,
    ]
);

use crate::{
    divider::Divider, empty::Empty, label::Label, stock::*, text_field::TextField,
    theme::ActiveTheme, StyledExt as _,
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
    fn selected_index(&self) -> usize;
    fn set_selected_index(&mut self, index: usize, cx: &mut ViewContext<Picker<Self>>);
    fn selected_index_changed(
        &self,
        _ix: usize,
        _cx: &mut ViewContext<Picker<Self>>,
    ) -> Option<Box<dyn Fn(&mut WindowContext) + 'static>> {
        None
    }

    fn confirm(&mut self, secondary: bool, cx: &mut ViewContext<Picker<Self>>) {}
    fn dismissed(&mut self, cx: &mut ViewContext<Picker<Self>>) {}
    fn should_dismiss(&self) -> bool {
        return true;
    }
    fn render_query(&self, input: &View<TextField>, _cx: &mut ViewContext<Picker<Self>>) -> Div {
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
    fn render_match(
        &self,
        ix: usize,
        selected: bool,
        cx: &mut ViewContext<Picker<Self>>,
    ) -> Option<Self::ListItem>;

    fn separators_after_indices(&self) -> Vec<usize> {
        Vec::new()
    }
    fn update_matches(&mut self, query: &str, cx: &mut ViewContext<Picker<Self>>) -> Task<()>;
    fn confirm_update_query(&mut self, _cx: &mut ViewContext<Picker<Self>>) -> Option<String> {
        None
    }
    fn finalize_update_matches(
        &mut self,
        _query: String,
        _duration: Duration,
        _cx: &mut ViewContext<Picker<Self>>,
    ) -> bool {
        false
    }
    fn selected_as_query(&self) -> Option<String> {
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
    query_input: Option<View<TextField>>,
    width: Option<Length>,
    max_height: Option<Length>,
    is_modal: bool,
    head: View<Empty>,
    pending_update_matches: Option<PendingUpdateMatches>,
    confirm_on_update: Option<bool>,
}

impl<D: PickerDelegate> Picker<D> {
    fn new(
        delegate: D,
        kind: ContainerKind,
        query_input: Option<View<TextField>>,
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
            query_input: None,
            head: cx.new_view(Empty::new),
            width: None,
            is_modal: false,
            max_height: Some(rems(20.).into()),
            element_container,
            pending_update_matches: None,
            confirm_on_update: None,
        }
    }

    fn new_query_input(
        placehoder: impl Into<SharedString>,
        cx: &mut ViewContext<Self>,
    ) -> View<TextField> {
        cx.new_view(|cx| {
            let mut input = TextField::new(cx);
            input.set_placeholder(placehoder, cx);
            input
        })
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

    pub fn set_query(&mut self, query: &str, cx: &mut ViewContext<Self>) {
        if let Some(input) = &self.query_input {
            input.update(cx, |this, cx| this.set_text(query, cx));
        }
    }

    /// Return the query input string.
    pub fn query(&self, cx: &AppContext) -> String {
        if let Some(input) = &self.query_input {
            input.read(cx).text(cx)
        } else {
            String::new()
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
            .py_2()
            .track_scroll(scroll_handle.clone())
            .into_any_element(),
            ElementContainer::List(state) => list(state.clone())
                .with_sizing_behavior(sizing_behavior)
                .flex_grow()
                .py_2()
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
                    .render_match(ix, ix == self.delegate.selected_index(), cx),
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

    fn on_click(&mut self, ix: usize, secondary: bool, cx: &mut ViewContext<Self>) {
        cx.stop_propagation();
        cx.prevent_default();
        self.set_selected_index(ix, false, cx);
        self.do_confirm(secondary, cx)
    }

    fn do_confirm(&mut self, secondary: bool, cx: &mut ViewContext<Self>) {
        // if let Some(update_query) = self.delegate.confirm_update_query(cx) {
        //     self.set_query(update_query, cx);
        //     self.delegate.set_selected_index(0, cx);
        // } else {
        self.delegate.confirm(secondary, cx)
        // }
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
        if self.pending_update_matches.is_some()
            && !self
                .delegate
                .finalize_update_matches(self.query(cx), Duration::from_millis(16), cx)
        {
            self.confirm_on_update = Some(false)
        } else {
            self.pending_update_matches.take();
            self.do_confirm(false, cx);
        }
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

    fn use_selected_query(&mut self, _: &UseSelectedQuery, cx: &mut ViewContext<Self>) {
        if let Some(new_query) = self.delegate.selected_as_query() {
            self.set_query(&new_query, cx);
            cx.stop_propagation();
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
}

impl<D: PickerDelegate> Render for Picker<D> {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        v_flex()
            .key_context("Picker")
            .size_full()
            .when_some(self.width, |el, width| el.w(width))
            .overflow_hidden()
            .when(self.is_modal, |this| this.elevation_3(cx))
            .on_action(cx.listener(Self::select_next))
            .on_action(cx.listener(Self::select_prev))
            .on_action(cx.listener(Self::select_first))
            .on_action(cx.listener(Self::select_last))
            .on_action(cx.listener(Self::cancel))
            .on_action(cx.listener(Self::confirm))
            // .on_action(cx.listener(Self::secondary_confirm))
            .on_action(cx.listener(Self::use_selected_query))
            // .on_action(cx.listener(Self::confirm_input))
            .child(match self.query_input {
                Some(ref input) => self.delegate.render_query(input, cx),
                None => div().child(self.head.clone()),
            })
            .when(self.delegate.match_count() > 0, |el| {
                el.child(
                    v_flex()
                        .flex_grow()
                        .when_some(self.max_height, |div, max_h| div.max_h(max_h))
                        .overflow_hidden()
                        // .children(self.delegate.render_header(cx))
                        .child(self.render_element_container(cx)),
                )
            })
            .when(self.delegate.match_count() == 0, |el| {
                el.child(
                    v_flex()
                        .flex_grow()
                        .py_2()
                        .child(div().child(Label::new("No matched.").text_color(cx.theme().muted))),
                )
            })
    }
}
