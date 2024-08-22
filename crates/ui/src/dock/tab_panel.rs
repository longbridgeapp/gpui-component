use std::sync::Arc;

use gpui::{
    div, prelude::FluentBuilder, rems, AppContext, DefiniteLength, DragMoveEvent, Empty,
    FocusHandle, FocusableView, InteractiveElement as _, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, View, ViewContext, VisualContext as _,
};

use crate::{
    button::Button,
    h_flex,
    tab::{Tab, TabBar},
    theme::ActiveTheme,
    v_flex, AxisExt, IconName, Placement, Selectable, Sizable,
};

use super::{Panel, PanelView, StackPanel};

#[derive(Clone)]
pub(crate) struct DragPanel {
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
            will_split_placement: None,
        }
    }

    pub(super) fn set_parent(&mut self, parent: View<StackPanel>) {
        self.stack_panel = Some(parent);
    }

    /// Return current active_panel View
    pub fn active_panel(&self) -> Option<Arc<dyn PanelView>> {
        self.panels.get(self.active_ix).cloned()
    }

    /// Add a panel to the end of the tabs
    pub fn add_panel(&mut self, panel: Arc<dyn PanelView>, cx: &mut ViewContext<Self>) {
        if self
            .panels
            .iter()
            .any(|p| p.view().entity_id() == panel.view().entity_id())
        {
            return;
        }

        self.panels.push(panel);
        // set the active panel to the new panel
        self.active_ix = self.panels.len() - 1;
        cx.notify();
    }

    /// Remove a panel from the tab panel
    pub fn remove_panel(&mut self, panel: Arc<dyn PanelView>, cx: &mut ViewContext<Self>) {
        self.detach_panel(panel, cx);
        self.remove_self_if_empty(cx)
    }

    fn detach_panel(&mut self, panel: Arc<dyn PanelView>, _cx: &mut ViewContext<Self>) {
        let panel_view = panel.view();
        self.panels.retain(|p| p.view() != panel_view);
        if self.active_ix >= self.panels.len() {
            self.active_ix = self.panels.len().saturating_sub(1);
        }
    }

    /// Check to remove self from the parent StackPanel, if there is no panel left
    fn remove_self_if_empty(&self, cx: &mut ViewContext<Self>) {
        if !self.panels.is_empty() {
            return;
        }

        let tab_view = cx.view().clone();
        if let Some(stack_panel) = self.stack_panel.as_ref() {
            stack_panel.update(cx, |view, cx| {
                view.remove_panel(tab_view, cx);
            })
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
        self.active_panel()
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
                            .bg(cx.theme().drop_target)
                            .map(|this| match self.will_split_placement {
                                Some(placement) => {
                                    let size = DefiniteLength::Fraction(0.25);
                                    match placement {
                                        Placement::Left => this.left_0().top_0().bottom_0().w(size),
                                        Placement::Right => {
                                            this.right_0().top_0().bottom_0().w(size)
                                        }
                                        Placement::Top => this.top_0().left_0().right_0().h(size),
                                        Placement::Bottom => {
                                            this.bottom_0().left_0().right_0().h(size)
                                        }
                                    }
                                }
                                None => this.top_0().left_0().size_full(),
                            })
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
            // center to merge into the current tab
            self.will_split_placement = None;
        }
        cx.notify()
    }

    fn on_drop(&mut self, drag: &DragPanel, cx: &mut ViewContext<Self>) {
        let panel = drag.panel.clone();
        let is_same_tab = drag.tab_panel == *cx.view();

        // If target is same tab, and it is only one panel, do nothing.
        if is_same_tab {
            if self.will_split_placement.is_none() {
                return;
            } else {
                if self.panels.len() == 1 {
                    return;
                }
            }
        }

        // Here is looks like remove_panel on a same item, but it differnece.
        //
        // We must to split it to remove_panel, unless it will be crash by error:
        // Cannot update ui::dock::tab_panel::TabPanel while it is already being updated
        if is_same_tab {
            self.detach_panel(panel.clone(), cx);
        } else {
            let _ = drag.tab_panel.update(cx, |view, cx| {
                view.detach_panel(panel.clone(), cx);
                view.remove_self_if_empty(cx);
            });
        }

        // Insert into new tabs
        if let Some(placement) = self.will_split_placement {
            self.split_panel(panel, placement, cx);
        } else {
            self.add_panel(panel, cx);
        }

        self.remove_self_if_empty(cx);
    }

    /// Add panel with split placement
    fn split_panel(
        &self,
        panel: Arc<dyn PanelView>,
        placement: Placement,
        cx: &mut ViewContext<Self>,
    ) {
        // wrap the panel in a TabPanel
        let new_tab_panel = cx.new_view(|cx| Self::new(cx));
        new_tab_panel.update(cx, |view, cx| {
            view.add_panel(panel, cx);
        });

        let stack_panel = self.stack_panel.as_ref().unwrap();
        let parent_axis = stack_panel.read(cx).axis;
        let ix = stack_panel
            .read(cx)
            .index_of_panel(cx.view().clone())
            .unwrap_or_default();

        if parent_axis.is_vertical() && placement.is_vertical() {
            stack_panel.update(cx, |view, cx| {
                view.add_panel_at(new_tab_panel, ix, placement, cx);
            });
        } else if parent_axis.is_horizontal() && placement.is_horizontal() {
            stack_panel.update(cx, |view, cx| {
                view.add_panel_at(new_tab_panel, ix, placement, cx);
            });
        } else {
            // 1. Create new StackPanel with new axis
            // 2. Move cx.view() from parent StackPanel to the new StackPanel
            // 3. Add the new TabPanel to the new StackPanel at the correct index
            // 4. Add new StackPanel to the parent StackPanel at the correct index
            let tab_panel = cx.view().clone();

            // Try to use the old stack panel, not just create a new one, to avoid too many nested stack panels
            let new_stack_panel = if stack_panel.read(cx).panels_len() <= 1 {
                stack_panel.update(cx, |view, cx| {
                    view.remove_all_panels(cx);
                    view.set_axis(placement.axis(), cx);
                });
                stack_panel.clone()
            } else {
                cx.new_view(|cx| {
                    let mut panel = StackPanel::new(placement.axis(), cx);
                    panel.parent = Some(stack_panel.clone());
                    panel
                })
            };

            new_stack_panel.update(cx, |view, cx| match placement {
                Placement::Left | Placement::Top => {
                    view.add_panel(new_tab_panel, None, cx);
                    view.add_panel(tab_panel.clone(), None, cx);
                }
                Placement::Right | Placement::Bottom => {
                    view.add_panel(tab_panel.clone(), None, cx);
                    view.add_panel(new_tab_panel, None, cx);
                }
            });

            if *stack_panel != new_stack_panel {
                stack_panel.update(cx, |view, cx| {
                    view.replace_panel(tab_panel.clone(), new_stack_panel.clone(), cx);
                });
            }

            cx.spawn(|_, mut cx| async move {
                cx.update(|cx| tab_panel.update(cx, |view, cx| view.remove_self_if_empty(cx)))
            })
            .detach()
        }
    }
}

impl Panel for TabPanel {}

impl FocusableView for TabPanel {
    fn focus_handle(&self, _cx: &AppContext) -> gpui::FocusHandle {
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
