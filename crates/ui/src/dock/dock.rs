use gpui::{
    div, rems, Bounds, Context, ElementId, Empty, IntoElement, Model, ParentElement as _, Pixels,
    Render, RenderOnce, StatefulInteractiveElement as _, Styled as _, ViewContext,
};

use crate::{
    button::Button, dock::Node, h_flex, tab::TabBar, v_flex, IconName, Placement, Selectable as _,
    Sizable as _,
};

use super::{DockState, NodeIndex, Panel, SurfaceIndex};

/// What directions can this dock be split in?
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum AllowedSplits {
    #[default]
    /// Allow splits in any direction (horizontal and vertical).
    All = 0b11,

    /// Only allow split in a horizontal directions.
    LeftRightOnly = 0b10,

    /// Only allow splits in a vertical directions.
    TopBottomOnly = 0b01,

    /// Don't allow splits at all.
    None = 0b00,
}

impl std::ops::BitAnd for AllowedSplits {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self::from_u8(self as u8 & rhs as u8)
    }
}

/// An enum expressing an entry in the `to_remove` field in [`DockArea`].
#[derive(Debug, Clone, Copy)]
pub(super) enum TabRemoval {
    Node(SurfaceIndex, NodeIndex, usize),
    Window(SurfaceIndex),
}

impl From<SurfaceIndex> for TabRemoval {
    fn from(index: SurfaceIndex) -> Self {
        TabRemoval::Window(index)
    }
}

impl From<(SurfaceIndex, NodeIndex, usize)> for TabRemoval {
    fn from((si, ni, ti): (SurfaceIndex, NodeIndex, usize)) -> TabRemoval {
        TabRemoval::Node(si, ni, ti)
    }
}

impl AllowedSplits {
    /// Create allowed splits from a u8, panics if an invalid value is given.
    #[inline(always)]
    fn from_u8(u8: u8) -> Self {
        match u8 {
            0b11 => AllowedSplits::All,
            0b10 => AllowedSplits::LeftRightOnly,
            0b01 => AllowedSplits::TopBottomOnly,
            0b00 => AllowedSplits::None,
            _ => panic!("Provided an invalid value for allowed splits: {u8:0x}"),
        }
    }
}

pub struct DockArea<Tab: Panel> {
    id: ElementId,
    dock_state: DockState<Tab>,
    draggable_tabs: bool,
    allowed_splits: AllowedSplits,
    window_bounds: Option<Bounds<Pixels>>,

    to_remove: Vec<TabRemoval>,
    to_detach: Vec<(SurfaceIndex, NodeIndex, usize)>,
    new_focused: Option<(SurfaceIndex, NodeIndex)>,
    // tab_hover_rect: Option<(Rect, TabIndex)>,
}

impl<Tab: Panel> DockArea<Tab> {
    pub fn new(
        id: impl Into<ElementId>,
        dock_state: DockState<Tab>,
        cx: &mut ViewContext<Self>,
    ) -> Self {
        Self {
            id: id.into(),
            dock_state,
            draggable_tabs: true,
            allowed_splits: AllowedSplits::All,
            window_bounds: None,
            to_remove: Vec::new(),
            to_detach: Vec::new(),
            new_focused: None,
        }
    }

    fn render_tabs(
        &self,
        (surface_index, node_index): (SurfaceIndex, NodeIndex),
        cx: &mut ViewContext<Self>,
    ) -> impl IntoElement {
        assert!(self.dock_state[surface_index][node_index].is_leaf());

        let focused = self.dock_state.focused_leaf();
        let tabs = self.dock_state[surface_index][node_index]
            .tabs()
            .expect("This node must be a leaf here");

        if tabs.len() == 1 {
            let tab = tabs.get(0).unwrap();
            if let Some(title) = tab.title(cx) {
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
                tabs.iter()
                    .enumerate()
                    .map(|(ix, tab)| {
                        let active =
                            self.dock_state[surface_index][node_index].active_tab_ix() == Some(ix);
                        let title = tab.title(cx).unwrap_or("Unnamed".into());

                        crate::tab::Tab::new(("tab", ix), title)
                            .selected(active)
                            .on_click(cx.listener(move |view, _, cx| {
                                view.dock_state[surface_index][node_index].set_active_tab(ix);
                            }))
                    })
                    .collect::<Vec<_>>(),
            )
            .into_any_element()
    }

    fn render_nodes(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let surf_index = SurfaceIndex::main();

        for node_index in self.dock_state[surf_index].breadth_first_index_iter() {
            if self.dock_state[surf_index][node_index].is_parent() {
                // self.compute_rect_sizes(ui, (surf_index, node_index), max_rect);
            }
        }

        div().children(
            self.dock_state[surf_index]
                .breadth_first_index_iter()
                .filter_map(|node_index| {
                    if self.dock_state[surf_index][node_index].is_leaf() {
                        Some(self.render_leaf((surf_index, node_index), cx))
                    } else {
                        None
                    }
                }),
        )
    }

    fn compute_rect_sizes(
        &mut self,
        (surface_index, node_index): (SurfaceIndex, NodeIndex),
        max_bounds: Bounds<Pixels>,
    ) {
    }

    fn render_leaf(
        &mut self,
        (surface_index, node_index): (SurfaceIndex, NodeIndex),
        cx: &mut ViewContext<Self>,
    ) -> impl IntoElement {
        let bounds = self.dock_state[surface_index][node_index].bounds();

        v_flex()
            .child(self.render_tabs((surface_index, node_index), cx))
            .child(self.render_tab_body((surface_index, node_index), cx))
    }

    fn render_tab_body(
        &self,
        (surface_index, node_index): (SurfaceIndex, NodeIndex),
        cx: &mut ViewContext<Self>,
    ) -> impl IntoElement {
        let tabs = self.dock_state[surface_index][node_index]
            .tabs()
            .expect("This node must be a leaf here");

        let active_tab_ix = self.dock_state[surface_index][node_index].active_tab_ix();
        let active_tab = active_tab_ix.map(|ix| tabs.get(ix).unwrap());

        // if let Some(tab) = active_tab {
        //     tab.title(cx)
        // } else {
        Empty {}.into_any_element()
        // }
    }
}

impl<Tab: Panel> Render for DockArea<Tab> {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        v_flex()
    }
}
