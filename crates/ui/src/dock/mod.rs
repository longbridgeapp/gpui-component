mod panel;
mod stack_panel;
mod tab_panel;

use std::{fmt::Display, rc::Rc};

use anyhow::Result;
use gpui::{
    actions, div, prelude::FluentBuilder, AnyWeakView, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, SharedString, Styled, Task, View, ViewContext,
};
pub use panel::*;
use serde::{Deserialize, Serialize};
pub use stack_panel::*;
pub use tab_panel::*;
use uuid::Uuid;

actions!(dock, [ToggleZoom, ClosePanel]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanelId(Uuid);

impl PanelId {
    /// Creates a new panel ID with unique UUID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Display for PanelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The main area of the dock.
pub struct DockArea {
    id: SharedString,
    root: View<StackPanel>,
    zoom_view: Option<AnyWeakView>,
    /// Read state for the DockArea
    read_state: Option<Rc<dyn Fn(&str) -> Task<Result<String>>>>,
    /// Write state for the DockArea
    write_state: Option<Rc<dyn Fn(&str, &str) -> Task<Result<()>>>>,
}

impl DockArea {
    pub fn new(
        id: impl Into<SharedString>,
        root: View<StackPanel>,
        _cx: &mut ViewContext<Self>,
    ) -> Self {
        Self {
            id: id.into(),
            root,
            zoom_view: None,
            read_state: None,
            write_state: None,
        }
    }

    /// Returns the ID of the dock area.
    pub fn id(&self) -> SharedString {
        self.id.clone()
    }

    /// Toggles the zoom view.
    pub fn toggle_zoom<P: Panel>(&mut self, panel: View<P>, cx: &mut ViewContext<Self>) {
        if self.zoom_view.is_some() {
            self.zoom_view = None;
        } else {
            self.zoom_view = Some(panel.downgrade().into());
        }
        cx.notify();
    }

    /// Returns the root stack panel.
    pub fn root(&self) -> View<StackPanel> {
        self.root.clone()
    }

    /// Return the index of the panel.
    pub fn index_of_panel<P: Panel>(
        &self,
        panel: View<P>,
        cx: &mut ViewContext<Self>,
    ) -> Option<usize> {
        self.root.read(cx).index_of_panel(panel)
    }

    /// Return the existing panel by type.
    pub fn panel<P: Panel>(&self, cx: &mut ViewContext<Self>) -> Option<View<P>> {
        self.root.read(cx).panel::<P>(cx)
    }

    /// Save the layout.
    pub fn with_read_state<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> Task<Result<String>> + 'static,
    {
        self.read_state = Some(Rc::new(f));
        self
    }

    pub fn with_write_state<F>(mut self, f: F) -> Self
    where
        F: Fn(&str, &str) -> Task<Result<()>> + 'static,
    {
        self.write_state = Some(Rc::new(f));
        self
    }

    pub fn read_state(&self, key: &str) -> Task<Result<String>> {
        self.read_state
            .as_ref()
            .map(|f| f(key))
            .unwrap_or_else(|| Task::Ready(None))
    }

    pub fn write_state(&self, key: &str, value: &str) -> Task<Result<()>> {
        self.write_state
            .as_ref()
            .map(|f| f(key, value))
            .unwrap_or_else(|| Task::Ready(None))
    }
}

impl Render for DockArea {
    fn render(&mut self, _: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .id("dock-area")
            .size_full()
            .overflow_hidden()
            .map(|this| {
                if let Some(zoom_view) = self.zoom_view.as_ref().and_then(|view| view.upgrade()) {
                    this.child(zoom_view)
                } else {
                    this.child(self.root.clone())
                }
            })
    }
}
