use std::sync::Arc;

use gpui::{
    div, px, rems, AppContext, Empty, FocusHandle, FocusableView, InteractiveElement as _,
    IntoElement, ParentElement, Pixels, Render, StatefulInteractiveElement, Styled, View,
    ViewContext, WindowContext,
};

use crate::{
    button::Button,
    h_flex,
    tab::{Tab, TabBar},
    theme::ActiveTheme,
    v_flex, IconName, Placement, Selectable, Sizable, StyledExt,
};

use super::{Panel, PanelView};

pub struct TabPanel {
    focus_handle: FocusHandle,
    panels: Vec<Arc<dyn PanelView>>,
    active_ix: usize,
    placement: Placement,
    size: Pixels,
}

impl TabPanel {
    pub fn new(cx: &mut ViewContext<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            panels: Vec::new(),
            active_ix: 0,
            placement: Placement::Left,
            size: px(50.),
        }
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

    fn render_tabs(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        if self.panels.len() == 1 {
            let panel = self.panels.get(0).unwrap();
            if let Some(title) = panel.title(cx) {
                return h_flex()
                    .justify_between()
                    .items_center()
                    .py_2()
                    .px_3()
                    .line_height(rems(1.0))
                    .child(title)
                    .child(
                        Button::new("menu", cx)
                            .icon(IconName::Ellipsis)
                            .xsmall()
                            .ghost(),
                    )
                    .into_any_element();
            }

            return Empty {}.into_any_element();
        }

        TabBar::new("tabs")
            .children(
                self.panels
                    .iter()
                    .enumerate()
                    .map(|(ix, panel)| {
                        let active = ix == self.active_ix;
                        let title = panel.title(cx).unwrap_or("Unnamed".into());

                        Tab::new(("tab", ix), title)
                            .selected(active)
                            .on_click(cx.listener(move |view, _, cx| {
                                view.active_ix = ix;
                            }))
                    })
                    .collect::<Vec<_>>(),
            )
            .into_any_element()
    }

    fn render_active_panel(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        self.active_panel(cx)
            .map(|panel| {
                h_flex()
                    .id("tab-content")
                    .overflow_y_scroll()
                    .flex_1()
                    .child(panel.into_any())
                    .into_any_element()
            })
            .unwrap_or(Empty {}.into_any_element())
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
