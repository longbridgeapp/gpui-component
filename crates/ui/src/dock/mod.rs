mod panel;
mod stack_panel;
mod tab_panel;

use std::sync::Arc;

use gpui::{
    actions, div, prelude::FluentBuilder, AnyElement, AnyView, Axis, InteractiveElement as _,
    IntoElement, ParentElement as _, Pixels, Render, SharedString, Styled, View, ViewContext,
    VisualContext, WindowContext,
};
pub use panel::*;
pub use stack_panel::*;
pub use tab_panel::*;

actions!(dock, [ToggleZoom, ClosePanel]);

/// The main area of the dock.
pub struct DockArea {
    id: SharedString,
    items: DockItem,
    zoom_view: Option<AnyView>,
}

/// DockItem is a tree structure that represents the layout of the dock.
#[derive(Clone)]
pub enum DockItem {
    Split {
        axis: gpui::Axis,
        items: Vec<DockItem>,
        sizes: Vec<Option<Pixels>>,
        view: View<StackPanel>,
    },
    Tabs {
        items: Vec<DockItem>,
        active_ix: usize,
        view: View<TabPanel>,
    },
    Panel {
        view: Arc<dyn PanelView>,
    },
}

impl DockItem {
    pub fn split(
        axis: Axis,
        items: Vec<DockItem>,
        sizes: Vec<Option<Pixels>>,
        dock_area: &View<DockArea>,
        cx: &mut WindowContext,
    ) -> Self {
        let stack_panel = cx.new_view(|cx| {
            let mut stack_panel = StackPanel::new(axis, cx);

            for (i, item) in items.iter().enumerate() {
                let item = match item {
                    DockItem::Panel { .. } => Self::tabs(vec![item.clone()], None, &dock_area, cx),
                    _ => item.clone(),
                };

                let view = item.view();
                let size = *sizes.get(i).unwrap();
                stack_panel.add_panel(view.clone(), size, dock_area.downgrade(), cx)
            }

            stack_panel
        });
        Self::Split {
            axis,
            items,
            sizes,
            view: stack_panel,
        }
    }

    pub fn tabs(
        items: Vec<DockItem>,
        active_ix: Option<usize>,
        dock_area: &View<DockArea>,
        cx: &mut WindowContext,
    ) -> Self {
        let active_ix = active_ix.unwrap_or(0);
        let tab_panel = cx.new_view(|cx| {
            let mut tab_panel = TabPanel::new(None, dock_area.downgrade(), cx);

            for item in items.iter() {
                let view = item.view();
                tab_panel.add_panel(view, cx)
            }

            tab_panel
        });

        Self::Tabs {
            items,
            active_ix,
            view: tab_panel,
        }
    }

    pub fn panel(view: impl PanelView + 'static) -> Self {
        Self::Panel {
            view: Arc::new(view),
        }
    }

    /// Returns the views of the dock item.
    fn view(&self) -> Arc<dyn PanelView> {
        match self {
            Self::Split { view, .. } => Arc::new(view.clone()),
            Self::Tabs { view, .. } => Arc::new(view.clone()),
            Self::Panel { view } => view.clone(),
        }
    }
}

impl DockArea {
    pub fn new(id: impl Into<SharedString>, cx: &mut WindowContext) -> Self {
        let stack_panel = cx.new_view(|cx| StackPanel::new(Axis::Horizontal, cx));
        let dock_item = DockItem::Split {
            axis: Axis::Horizontal,
            items: vec![],
            sizes: vec![],
            view: stack_panel.clone(),
        };

        Self {
            id: id.into(),
            items: dock_item,
            zoom_view: None,
        }
    }

    pub fn set_root(&mut self, item: DockItem, cx: &mut ViewContext<Self>) {
        self.items = item;
        cx.notify();
    }

    // pub fn add_item(&mut self, item: DockItem, cx: &mut ViewContext<Self>) {
    //     match &mut self.items {
    //         DockItem::Split { items, view, .. } => {
    //             items.push(item.clone());
    //             view.update(cx, |view, cx| {
    //                 view.panels.push(item.view());
    //             });
    //         }
    //         DockItem::Tabs { items, view, .. } => {
    //             items.push(item.clone());
    //             view.update(cx, |view, cx| {
    //                 view.
    //             });
    //         }
    //         DockItem::Panel { view, .. } => {
    //             let old_item = self.items.clone();
    //             self.items = DockItem::split(
    //                 Axis::Horizontal,
    //                 vec![old_item, item],
    //                 vec![None, None],
    //                 cx.view(),
    //                 cx,
    //             );
    //         }
    //     }
    // }

    /// Returns the ID of the dock area.
    pub fn id(&self) -> SharedString {
        self.id.clone()
    }

    pub fn set_zoomed_in<P: Panel>(&mut self, panel: View<P>, cx: &mut ViewContext<Self>) {
        self.zoom_view = Some(panel.into());
        cx.notify();
    }

    pub fn set_zoomed_out(&mut self, cx: &mut ViewContext<Self>) {
        self.zoom_view = None;
        cx.notify();
    }

    fn render_items(&self, _cx: &mut ViewContext<Self>) -> AnyElement {
        match &self.items {
            DockItem::Split { view, .. } => view.clone().into_any_element(),
            DockItem::Tabs { view, .. } => view.clone().into_any_element(),
            DockItem::Panel { view } => view.view().into_any_element(),
        }
    }
}

impl Render for DockArea {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        // println!("Rendering dock area");
        div()
            .id("dock-area")
            .size_full()
            .overflow_hidden()
            .map(|this| {
                if let Some(zoom_view) = self.zoom_view.clone() {
                    this.child(zoom_view)
                } else {
                    this.child(self.render_items(cx))
                }
            })
    }
}
