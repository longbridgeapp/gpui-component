use std::{cell::Cell, rc::Rc, time::Duration};

use gpui::prelude::FluentBuilder as _;

use crate::input::{TextEvent, TextInput};
use crate::theme::{ActiveTheme, Colorize as _};
use crate::{scrollbar::Scrollbar, v_flex};
use crate::{Icon, IconName};
use gpui::{
    actions, div, px, uniform_list, AppContext, FocusHandle, FocusableView,
    InteractiveElement as _, IntoElement, KeyBinding, Length, ListSizingBehavior, MouseButton,
    ParentElement as _, Render, StatefulInteractiveElement as _, Styled as _, Task,
    UniformListScrollHandle, View, ViewContext, VisualContext as _,
};

actions!(list, [Cancel, Confirm, SelectPrev, SelectNext]);

pub fn init(cx: &mut AppContext) {
    let context: Option<&str> = Some("List");
    cx.bind_keys([
        KeyBinding::new("escape", Cancel, context),
        KeyBinding::new("enter", Confirm, context),
        KeyBinding::new("up", SelectPrev, context),
        KeyBinding::new("down", SelectNext, context),
    ]);
}

#[allow(unused)]
pub trait ListDelegate: Sized + 'static {
    type Item: IntoElement;

    fn perform_search(&mut self, query: &str, cx: &mut ViewContext<List<Self>>) {}

    /// Return the number of items in the list.
    fn items_count(&self) -> usize;
    fn render_item(&self, ix: usize, cx: &mut ViewContext<List<Self>>) -> Option<Self::Item>;

    /// Return the confirmed index of the selected item.
    fn confirmed_index(&self) -> Option<usize> {
        None
    }

    /// Set the confirm and give the selected index.
    fn confirm(&mut self, ix: Option<usize>, cx: &mut ViewContext<List<Self>>) {}
    fn cancel(&mut self, cx: &mut ViewContext<List<Self>>) {}
}

pub struct List<D: ListDelegate> {
    focus_handle: FocusHandle,
    delegate: D,
    max_height: Option<Length>,
    query_input: Option<View<TextInput>>,

    enable_scrollbar: bool,
    vertical_scroll_handle: UniformListScrollHandle,
    scrollbar_drag_state: Rc<Cell<Option<f32>>>,
    show_scrollbar: bool,
    hide_scrollbar_task: Option<Task<()>>,

    selected_index: Option<usize>,
}

impl<D> List<D>
where
    D: ListDelegate,
{
    pub fn new(delegate: D, cx: &mut ViewContext<Self>) -> Self {
        let query_input = cx.new_view(|cx| {
            TextInput::new(cx)
                .appearance(false)
                .prefix(Icon::new(IconName::Search).view(cx))
                .placeholder("Search...")
        });

        cx.subscribe(&query_input, Self::on_query_input_event)
            .detach();

        Self {
            focus_handle: cx.focus_handle(),
            delegate,
            query_input: Some(query_input),
            selected_index: None,
            vertical_scroll_handle: UniformListScrollHandle::new(),
            scrollbar_drag_state: Rc::new(Cell::new(None)),
            show_scrollbar: false,
            hide_scrollbar_task: None,
            max_height: None,
            enable_scrollbar: true,
        }
    }

    pub fn max_h(mut self, height: impl Into<Length>) -> Self {
        self.max_height = Some(height.into());
        self
    }

    pub fn no_scrollbar(mut self) -> Self {
        self.enable_scrollbar = false;
        self
    }

    pub fn no_query(mut self) -> Self {
        self.query_input = None;
        self
    }

    pub fn delegate(&self) -> &D {
        &self.delegate
    }

    pub fn delegate_mut(&mut self) -> &mut D {
        &mut self.delegate
    }

    pub fn focus(&mut self, cx: &mut ViewContext<Self>) {
        cx.focus(&self.focus_handle);
    }

    fn render_scrollbar(&self, cx: &mut ViewContext<Self>) -> Option<impl IntoElement> {
        if !self.enable_scrollbar {
            return None;
        }
        if !self.show_scrollbar {
            return None;
        }

        Scrollbar::new(
            cx.view().clone(),
            self.vertical_scroll_handle.clone(),
            self.scrollbar_drag_state.clone(),
            self.delegate.items_count(),
            true,
        )
        .map(|bar| {
            div()
                .occlude()
                .absolute()
                .h_full()
                .left_auto()
                .top_0()
                .right_0()
                .w(px(bar.width()))
                .bottom_0()
                .child(bar)
        })
    }

    fn hide_scrollbar(&mut self, cx: &mut ViewContext<Self>) {
        self.show_scrollbar = false;
        self.hide_scrollbar_task = Some(cx.spawn(|this, mut cx| async move {
            cx.background_executor().timer(Duration::from_secs(1)).await;
            this.update(&mut cx, |this, cx| {
                this.show_scrollbar = false;
                cx.notify();
            })
            .ok();
        }))
    }

    fn on_hover_to_autohide_scrollbar(&mut self, hovered: &bool, cx: &mut ViewContext<Self>) {
        if !self.enable_scrollbar {
            return;
        }

        if *hovered {
            self.show_scrollbar = true;
            self.hide_scrollbar_task.take();
            cx.notify();
        } else if !self.focus_handle.is_focused(cx) {
            self.hide_scrollbar(cx);
        }
    }

    fn scroll_to_selected_item(&mut self, _cx: &mut ViewContext<Self>) {
        if let Some(ix) = self.selected_index {
            self.vertical_scroll_handle.scroll_to_item(ix);
        }
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
                self.delegate.perform_search(&text.trim(), cx);
                cx.notify()
            }
            TextEvent::PressEnter => self.action_confirm(&Confirm, cx),
        }
    }

    fn action_cancel(&mut self, _: &Cancel, cx: &mut ViewContext<Self>) {
        self.delegate.cancel(cx);
        cx.notify();
    }

    fn action_confirm(&mut self, _: &Confirm, cx: &mut ViewContext<Self>) {
        self.delegate.confirm(self.selected_index, cx);
        cx.notify();
    }

    fn action_select_prev(&mut self, _: &SelectPrev, cx: &mut ViewContext<Self>) {
        let selected_index = self.selected_index.unwrap_or(0);
        if selected_index > 0 {
            self.selected_index = Some(selected_index - 1);
        } else {
            self.selected_index = Some(self.delegate.items_count() - 1);
        }

        self.scroll_to_selected_item(cx);
        cx.notify();
    }

    fn action_select_next(&mut self, _: &SelectNext, cx: &mut ViewContext<Self>) {
        let selected_index = self.selected_index.unwrap_or(0);
        if selected_index < self.delegate.items_count() - 1 {
            self.selected_index = Some(selected_index + 1);
        } else {
            self.selected_index = Some(0);
        }

        self.scroll_to_selected_item(cx);
        cx.notify();
    }
}

impl<D> FocusableView for List<D>
where
    D: ListDelegate,
{
    fn focus_handle(&self, cx: &AppContext) -> FocusHandle {
        if let Some(query_input) = &self.query_input {
            query_input.focus_handle(cx)
        } else {
            self.focus_handle.clone()
        }
    }
}

impl<D> Render for List<D>
where
    D: ListDelegate,
{
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let view = cx.view().clone();
        let vertical_scroll_handle = self.vertical_scroll_handle.clone();
        let items_count = self.delegate.items_count();
        let sizing_behavior = if self.max_height.is_some() {
            ListSizingBehavior::Infer
        } else {
            ListSizingBehavior::Auto
        };

        let selected_bg = cx.theme().accent.opacity(0.8);

        v_flex()
            .key_context("List")
            .id("list")
            .track_focus(&self.focus_handle)
            .size_full()
            .overflow_hidden()
            .on_action(cx.listener(Self::action_cancel))
            .on_action(cx.listener(Self::action_confirm))
            .on_action(cx.listener(Self::action_select_next))
            .on_action(cx.listener(Self::action_select_prev))
            .on_hover(cx.listener(Self::on_hover_to_autohide_scrollbar))
            .when_some(self.query_input.clone(), |this, input| {
                this.child(
                    div()
                        .px_2()
                        .border_b_1()
                        .border_color(cx.theme().border)
                        .child(input),
                )
            })
            .child(
                v_flex()
                    .flex_grow()
                    .min_h(px(100.))
                    .when_some(self.max_height, |this, h| this.max_h(h))
                    .overflow_hidden()
                    .child(
                        uniform_list(view, "uniform-list", items_count, {
                            move |list, visible_range, cx| {
                                visible_range
                                    .map(|ix| {
                                        div()
                                            .id("list-item")
                                            .w_full()
                                            .children(list.delegate.render_item(ix, cx))
                                            .when_some(
                                                list.selected_index,
                                                |this, selected_index| {
                                                    this.when(ix == selected_index, |this| {
                                                        this.bg(selected_bg)
                                                    })
                                                },
                                            )
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(move |this, _, cx| {
                                                    this.selected_index = Some(ix);
                                                    this.action_confirm(&Confirm, cx);
                                                }),
                                            )
                                    })
                                    .collect::<Vec<_>>()
                            }
                        })
                        .flex_grow()
                        .with_sizing_behavior(sizing_behavior)
                        .track_scroll(vertical_scroll_handle)
                        .into_any_element(),
                    )
                    .children(self.render_scrollbar(cx)),
            )
    }
}
