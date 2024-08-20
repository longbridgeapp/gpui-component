use std::sync::Arc;

use gpui::{
    div, prelude::FluentBuilder, px, relative, rems, AppContext, DefiniteLength, DragMoveEvent,
    Empty, Entity, FocusHandle, FocusableView, InteractiveElement as _, IntoElement, ParentElement,
    Pixels, Render, StatefulInteractiveElement, Styled, View, ViewContext, VisualContext as _,
    WeakView, WindowContext,
};

use crate::{
    button::Button,
    h_flex,
    tab::{Tab, TabBar},
    theme::ActiveTheme,
    v_flex, IconName, Placement, Selectable, Sizable, StyledExt,
};

use super::{Panel, PanelView, StackPanel};

#[derive(Clone)]
pub(crate) struct DragPanel {
    pub(crate) ix: usize,
    pub(crate) panel: Arc<dyn PanelView>,
    pub(crate) tab_panel: View<TabPanel>,
}

impl Render for DragPanel {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .cursor_grab()
            .py_1()
            .px_3()
            .w_24()
            .overflow_hidden()
            .whitespace_nowrap()
            .border_1()
            .border_color(cx.theme().border)
            .rounded_md()
            .bg(cx.theme().tab_active)
            .shadow_md()
            .child(self.panel.title(cx))
    }
}

pub struct TabPanel {
    focus_handle: FocusHandle,
    stack_panel: Option<View<StackPanel>>,
    panels: Vec<Arc<dyn PanelView>>,
    active_ix: usize,
    placement: Placement,
    size: Pixels,

    /// When drag move, will get the placement of the panel to be split
    will_split_placement: Option<Placement>,
}

impl TabPanel {
    pub fn new(cx: &mut ViewContext<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            stack_panel: None,
            panels: Vec::new(),
            active_ix: 0,
            placement: Placement::Left,
            size: px(50.),
            will_split_placement: None,
        }
    }

    pub fn set_parent(&mut self, parent: View<StackPanel>) {
        self.stack_panel = Some(parent);
    }

    pub fn add_panel<D>(&mut self, panel: View<D>)
    where
        D: Panel,
    {
        self.panels.push(Arc::new(panel));
    }

    /// Return current active_panel View
    pub fn active_panel(&self, cx: &AppContext) -> Option<Arc<dyn PanelView>> {
        self.panels.get(self.active_ix).cloned()
    }

    fn remove_panel(&mut self, panel: &dyn PanelView, cx: &mut ViewContext<Self>) {
        let entity_id = panel.view().entity_id();

        self.panels.retain(|p| p.view().entity_id() != entity_id);
        if self.active_ix >= self.panels.len() {
            self.active_ix = self.panels.len().saturating_sub(1);
        }

        self.check_to_remove_self(cx)
    }

    /// Check to remove self from the parent StackPanel, if there is no panel left
    fn check_to_remove_self(&self, cx: &mut ViewContext<Self>) {
        let tab_view = cx.view().clone();
        if self.panels.is_empty() {
            if let Some(stack_panel) = self.stack_panel.as_ref() {
                stack_panel.update(cx, |view, cx| {
                    view.remove_panel(tab_view);
                })
            }
        }
    }

    fn render_tabs(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let view = cx.view().clone();

        if self.panels.len() == 1 {
            let panel = self.panels.get(0).unwrap();

            return h_flex()
                .id("tab")
                .justify_between()
                .items_center()
                .py_2()
                .px_3()
                .line_height(rems(1.0))
                .child(panel.title(cx))
                .child(
                    Button::new("menu", cx)
                        .icon(IconName::Ellipsis)
                        .xsmall()
                        .ghost(),
                )
                .on_drag(
                    DragPanel {
                        ix: 0,
                        panel: panel.clone(),
                        tab_panel: view,
                    },
                    |drag, cx| {
                        cx.stop_propagation();
                        cx.new_view(|_| drag.clone())
                    },
                )
                .into_any_element();
        }

        TabBar::new("tabs")
            .children(
                self.panels
                    .iter()
                    .enumerate()
                    .map(|(ix, panel)| {
                        let active = ix == self.active_ix;
                        Tab::new(("tab", ix), panel.title(cx))
                            .selected(active)
                            .on_click(cx.listener(move |view, _, _| {
                                view.active_ix = ix;
                            }))
                            .on_drag(
                                DragPanel {
                                    ix,
                                    panel: panel.clone(),
                                    tab_panel: view.clone(),
                                },
                                |drag, cx| {
                                    cx.stop_propagation();
                                    cx.new_view(|_| drag.clone())
                                },
                            )
                    })
                    .collect::<Vec<_>>(),
            )
            .into_any_element()
    }

    fn render_active_panel(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        self.active_panel(cx)
            .map(|panel| {
                div()
                    .id("tab-content")
                    .group("")
                    .overflow_y_scroll()
                    .flex_1()
                    .child(panel.view())
                    .on_drag_move(cx.listener(Self::on_panel_drag_move))
                    .child(
                        div()
                            .invisible()
                            .absolute()
                            .when_some(self.will_split_placement, |this, placement| {
                                let size = DefiniteLength::Fraction(0.25);
                                match placement {
                                    Placement::Left => this.left_0().top_0().bottom_0().w(size),
                                    Placement::Right => this.right_0().top_0().bottom_0().w(size),
                                    Placement::Top => this.top_0().left_0().right_0().h(size),
                                    Placement::Bottom => this.bottom_0().left_0().right_0().h(size),
                                }
                            })
                            .when(self.will_split_placement.is_none(), |this| {
                                this.top_0().left_0().size_full()
                            })
                            .bg(cx.theme().drop_target)
                            .group_drag_over::<DragPanel>("", |this| this.visible())
                            .on_drop(cx.listener(Self::on_drop)),
                    )
                    .into_any_element()
            })
            .unwrap_or(Empty {}.into_any_element())
    }

    /// Calculate the split direction based on the current mouse position
    fn on_panel_drag_move(&mut self, drag: &DragMoveEvent<DragPanel>, cx: &mut ViewContext<Self>) {
        let bounds = drag.bounds;
        let position = drag.event.position;

        // Check the mouse position to determine the split direction
        if position.x < bounds.left() + bounds.size.width * 0.25 {
            self.will_split_placement = Some(Placement::Left);
        } else if position.x > bounds.left() + bounds.size.width * 0.75 {
            self.will_split_placement = Some(Placement::Right);
        } else if position.y < bounds.top() + bounds.size.height * 0.25 {
            self.will_split_placement = Some(Placement::Top);
        } else if position.y > bounds.top() + bounds.size.height * 0.75 {
            self.will_split_placement = Some(Placement::Bottom);
        } else {
            self.will_split_placement = None;
        }
        cx.notify()
    }

    fn on_drop(&mut self, drag: &DragPanel, cx: &mut ViewContext<Self>) {
        if drag.tab_panel.entity_id() == cx.view().entity_id() {
            return;
        }

        let panel = drag.panel.clone();

        // Remove from old tabs
        let _ = drag.tab_panel.update(cx, |tab_panel, cx| {
            tab_panel.remove_panel(panel.as_ref(), cx);
        });

        // Insert into new tabs
        self.panels.push(drag.panel.clone());
        self.active_ix = self.panels.len() - 1;
        cx.notify()
    }
}

impl Panel for TabPanel {
    fn set_size(&mut self, size: Pixels, cx: &mut WindowContext) {
        self.size = size;
    }

    fn size(&self, cx: &WindowContext) -> Pixels {
        self.size
    }

    fn set_placement(&mut self, placement: Placement, cx: &mut WindowContext) {
        self.placement = placement;
    }

    fn placement(&self, cx: &WindowContext) -> Placement {
        self.placement
    }
}

impl FocusableView for TabPanel {
    fn focus_handle(&self, cx: &AppContext) -> gpui::FocusHandle {
        // FIXME: Delegate to the active panel
        self.focus_handle.clone()
    }
}

impl Render for TabPanel {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl gpui::IntoElement {
        v_flex()
            .size_full()
            .flex_none()
            .overflow_hidden()
            .bg(cx.theme().background)
            .child(self.render_tabs(cx))
            .child(self.render_active_panel(cx))
    }
}
