use std::sync::Arc;

use gpui::{
    deferred, div, prelude::FluentBuilder as _, px, AnyView, AppContext, Axis, Entity, EntityId,
    EventEmitter, FocusHandle, FocusableView, InteractiveElement as _, MouseButton, MouseDownEvent,
    MouseUpEvent, ParentElement as _, Pixels, Render, StatefulInteractiveElement, StyleRefinement,
    Styled as _, Subscription, View, ViewContext, VisualContext, WeakView, WindowContext,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ui::{theme::ActiveTheme, IconName, StyledExt as _};

const RESIZE_HANDLE_SIZE: Pixels = Pixels(6.);

use crate::{DraggedDock, Event};

use super::workspace::Workspace;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DockPosition {
    Left,
    Bottom,
    Right,
}

impl DockPosition {
    pub fn axis(&self) -> Axis {
        match self {
            Self::Left | Self::Right => Axis::Horizontal,
            Self::Bottom => Axis::Vertical,
        }
    }
}

pub enum PanelEvent {
    ZoomIn,
    ZoomOut,
    Activate,
    Close,
}

#[allow(unused)]
pub trait Panel: FocusableView + EventEmitter<PanelEvent> {
    fn persistent_name() -> &'static str;
    /// Return the position of the panel.
    fn position(&self, cx: &WindowContext) -> DockPosition;
    /// Return true if the panel can be positioned at the given position.
    fn can_position(&self, position: DockPosition) -> bool {
        true
    }
    /// Set the position of the panel.
    fn set_position(&mut self, position: DockPosition, cx: &mut ViewContext<Self>) {}
    /// Return the size of the panel.
    fn size(&self, cx: &WindowContext) -> Pixels;
    /// Set the size of the panel.
    fn set_size(&mut self, size: Option<Pixels>, cx: &mut ViewContext<Self>) {}
    /// Set the active state of the panel.
    fn set_active(&mut self, active: bool, cx: &mut ViewContext<Self>) {}
    fn icon(&self, _cx: &WindowContext) -> Option<IconName> {
        None
    }
    fn is_zoomed(&self, _cx: &WindowContext) -> bool {
        false
    }
    fn set_zoomed(&mut self, _zoomed: bool, _cx: &mut ViewContext<Self>) {}
    fn starts_open(&self, _cx: &WindowContext) -> bool {
        true
    }
}

pub trait PanelHandle: Send + Sync {
    fn id(&self) -> EntityId;
    fn persistent_name(&self) -> &'static str;
    fn position(&self, cx: &WindowContext) -> DockPosition;
    fn can_position(&self, position: DockPosition, cx: &WindowContext) -> bool;
    fn set_position(&self, position: DockPosition, cx: &mut WindowContext);
    fn size(&self, cx: &WindowContext) -> Pixels;
    fn set_size(&self, size: Option<Pixels>, cx: &mut WindowContext);
    fn icon(&self, cx: &WindowContext) -> Option<IconName>;
    fn focus_handle(&self, cx: &AppContext) -> FocusHandle;
    fn set_active(&self, active: bool, cx: &mut WindowContext);
    fn is_zoomed(&self, cx: &WindowContext) -> bool;
    fn set_zoomed(&self, zoomed: bool, cx: &mut WindowContext);
    fn to_any(&self) -> AnyView;
}

impl<T> PanelHandle for View<T>
where
    T: Panel,
{
    fn id(&self) -> EntityId {
        Entity::entity_id(self)
    }

    fn persistent_name(&self) -> &'static str {
        T::persistent_name()
    }

    fn position(&self, cx: &WindowContext) -> DockPosition {
        self.read(cx).position(cx)
    }

    fn can_position(&self, position: DockPosition, cx: &WindowContext) -> bool {
        self.read(cx).can_position(position)
    }

    fn set_position(&self, position: DockPosition, cx: &mut WindowContext) {
        self.update(cx, |this, cx| this.set_position(position, cx));
    }

    fn size(&self, cx: &WindowContext) -> Pixels {
        self.read(cx).size(cx)
    }

    fn set_size(&self, size: Option<Pixels>, cx: &mut WindowContext) {
        self.update(cx, |this, cx| this.set_size(size, cx));
    }

    fn icon(&self, cx: &WindowContext) -> Option<IconName> {
        self.read(cx).icon(cx)
    }

    fn focus_handle(&self, cx: &AppContext) -> FocusHandle {
        self.read(cx).focus_handle(cx).clone()
    }

    fn set_active(&self, active: bool, cx: &mut WindowContext) {
        self.update(cx, |this, cx| this.set_active(active, cx));
    }

    fn is_zoomed(&self, cx: &WindowContext) -> bool {
        self.read(cx).is_zoomed(cx)
    }

    fn set_zoomed(&self, zoomed: bool, cx: &mut WindowContext) {
        self.update(cx, |this, cx| this.set_zoomed(zoomed, cx));
    }

    fn to_any(&self) -> AnyView {
        self.clone().into()
    }
}

impl From<&dyn PanelHandle> for AnyView {
    fn from(handle: &dyn PanelHandle) -> Self {
        handle.to_any()
    }
}
struct PanelEntry {
    panel: Arc<dyn PanelHandle>,
    _subscriptions: [Subscription; 2],
}

pub struct Dock {
    position: DockPosition,
    panel_entries: Vec<PanelEntry>,
    is_open: bool,
    active_panel_index: usize,
    focus_handle: FocusHandle,
    resizeable: bool,
    _subscriptions: [Subscription; 1],
}

impl FocusableView for Dock {
    fn focus_handle(&self, _cx: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Dock {
    pub fn new(position: DockPosition, cx: &mut ViewContext<Workspace>) -> View<Self> {
        let focus_handle = cx.focus_handle();
        // let workspace = cx.view().clone();

        let dock = cx.new_view(|cx: &mut ViewContext<Self>| {
            let focus_subscription = cx.on_focus(&focus_handle, |dock, cx| {
                if let Some(active_entry) = dock.panel_entries.get(dock.active_panel_index) {
                    active_entry.panel.focus_handle(cx).focus(cx)
                }
            });

            Self {
                position,
                panel_entries: Vec::new(),
                active_panel_index: 0,
                is_open: false,
                focus_handle: focus_handle.clone(),
                resizeable: true,
                _subscriptions: [focus_subscription],
            }
        });

        cx.on_focus_in(&focus_handle, {
            let dock = dock.downgrade();
            move |_workspace, cx| {
                let Some(dock) = dock.upgrade() else {
                    return;
                };
                let Some(_panel) = dock.read(cx).active_panel() else {
                    return;
                };
            }
        })
        .detach();

        dock
    }

    pub fn position(&self) -> DockPosition {
        self.position
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn set_panel_zoomed(&mut self, panel: &AnyView, zoomed: bool, cx: &mut ViewContext<Self>) {
        for entry in &mut self.panel_entries {
            if entry.panel.id() == panel.entity_id() {
                if zoomed != entry.panel.is_zoomed(cx) {
                    entry.panel.set_zoomed(zoomed, cx);
                }
            } else if entry.panel.is_zoomed(cx) {
                entry.panel.set_zoomed(false, cx);
            }
        }

        cx.notify();
    }

    pub fn zoom_out(&mut self, cx: &mut ViewContext<Self>) {
        for entry in &mut self.panel_entries {
            if entry.panel.is_zoomed(cx) {
                entry.panel.set_zoomed(false, cx);
            }
        }
    }

    pub(crate) fn add_panel<T: Panel>(
        &mut self,
        panel: View<T>,
        workspace: WeakView<Workspace>,
        cx: &mut ViewContext<Self>,
    ) {
        let subscriptions = [
            cx.observe(&panel, |_, _, cx| cx.notify()),
            cx.subscribe(&panel, move |this, panel, event, cx| match event {
                PanelEvent::ZoomIn => {
                    this.set_panel_zoomed(&panel.to_any(), true, cx);
                    if !panel.focus_handle(cx).contains_focused(cx) {
                        cx.focus_view(&panel);
                    }
                    workspace
                        .update(cx, |workspace, cx| {
                            workspace.zoomed = Some(panel.downgrade().into());
                            workspace.zoomed_position = Some(panel.read(cx).position(cx));
                            cx.emit(Event::ZoomChanged);
                        })
                        .ok();
                }
                PanelEvent::ZoomOut => {
                    this.set_panel_zoomed(&panel.to_any(), false, cx);
                    workspace
                        .update(cx, |workspace, cx| {
                            if workspace.zoomed_position == Some(this.position) {
                                workspace.zoomed = None;
                                workspace.zoomed_position = None;
                                cx.emit(Event::ZoomChanged);
                            }
                            cx.notify();
                        })
                        .ok();
                }
                PanelEvent::Activate => {
                    if let Some(ix) = this
                        .panel_entries
                        .iter()
                        .position(|entry| entry.panel.id() == Entity::entity_id(&panel))
                    {
                        this.set_open(true, cx);
                        this.activate_panel(ix, cx);
                        cx.focus_view(&panel);
                    }
                }
                PanelEvent::Close => {
                    if this
                        .visible_panel()
                        .map_or(false, |p| p.id() == Entity::entity_id(&panel))
                    {
                        this.set_open(false, cx);
                    }
                }
            }),
        ];

        let _name = panel.persistent_name().to_string();

        self.panel_entries.push(PanelEntry {
            panel: Arc::new(panel.clone()),
            _subscriptions: subscriptions,
        });

        if panel.read(cx).starts_open(cx) {
            self.activate_panel(self.panel_entries.len() - 1, cx);
            self.set_open(true, cx);
        }

        cx.notify()
    }

    pub fn remove_panel<T: Panel>(&mut self, panel: &View<T>, cx: &mut ViewContext<Self>) {
        if let Some(panel_ix) = self
            .panel_entries
            .iter()
            .position(|entry| entry.panel.id() == Entity::entity_id(panel))
        {
            if panel_ix == self.active_panel_index {
                self.active_panel_index = 0;
                self.set_open(false, cx);
            } else if panel_ix < self.active_panel_index {
                self.active_panel_index -= 1;
            }
            self.panel_entries.remove(panel_ix);
            cx.notify();
        }
    }

    pub fn panels_len(&self) -> usize {
        self.panel_entries.len()
    }

    pub fn activate_panel(&mut self, panel_ix: usize, cx: &mut ViewContext<Self>) {
        if panel_ix != self.active_panel_index {
            if let Some(active_panel) = self.panel_entries.get(self.active_panel_index) {
                active_panel.panel.set_active(false, cx);
            }

            self.active_panel_index = panel_ix;
            if let Some(active_panel) = self.panel_entries.get(self.active_panel_index) {
                active_panel.panel.set_active(true, cx);
            }

            cx.notify();
        }
    }

    pub fn visible_panel(&self) -> Option<&Arc<dyn PanelHandle>> {
        let entry = self.visible_entry()?;
        Some(&entry.panel)
    }

    pub fn active_panel(&self) -> Option<&Arc<dyn PanelHandle>> {
        Some(&self.panel_entries.get(self.active_panel_index)?.panel)
    }

    pub fn active_panel_index(&self) -> usize {
        self.active_panel_index
    }

    fn visible_entry(&self) -> Option<&PanelEntry> {
        if self.is_open {
            self.panel_entries.get(self.active_panel_index)
        } else {
            None
        }
    }

    pub(crate) fn set_open(&mut self, open: bool, cx: &mut ViewContext<Self>) {
        if open != self.is_open {
            self.is_open = open;
            if let Some(active_panel) = self.panel_entries.get(self.active_panel_index) {
                active_panel.panel.set_active(open, cx);
            }

            cx.notify();
        }
    }

    pub fn panel<T: Panel>(&self) -> Option<View<T>> {
        self.panel_entries
            .iter()
            .find_map(|entry| entry.panel.to_any().clone().downcast().ok())
    }

    pub fn resize_active_panel(&mut self, size: Option<Pixels>, cx: &mut ViewContext<Self>) {
        if let Some(entry) = self.panel_entries.get_mut(self.active_panel_index) {
            let size = size.map(|size| size.max(RESIZE_HANDLE_SIZE).round());
            entry.panel.set_size(size, cx);
            cx.notify();
        }
    }
}

impl Render for Dock {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl gpui::IntoElement {
        if self.visible_entry().is_none() {
            return div().key_context("Dock").track_focus(&self.focus_handle);
        }

        let entry = self.visible_entry().unwrap();
        let size = entry.panel.size(cx);
        let position = self.position;
        let create_resize_handle = || {
            let handle = div()
                .id("resize-handle")
                .on_drag(DraggedDock(position), |dock, cx| {
                    cx.stop_propagation();
                    cx.new_view(|_| dock.clone())
                })
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|_, _: &MouseDownEvent, cx| {
                        cx.stop_propagation();
                    }),
                )
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(|v, e: &MouseUpEvent, cx| {
                        if e.click_count == 2 {
                            v.resize_active_panel(None, cx);
                            cx.stop_propagation();
                        }
                    }),
                )
                .occlude();

            match self.position() {
                DockPosition::Left => deferred(
                    handle
                        .absolute()
                        .right(-RESIZE_HANDLE_SIZE / 2.)
                        .top(px(0.))
                        .h_full()
                        .w(RESIZE_HANDLE_SIZE)
                        .cursor_col_resize(),
                ),
                DockPosition::Bottom => deferred(
                    handle
                        .absolute()
                        .top(-RESIZE_HANDLE_SIZE / 2.)
                        .left(px(0.))
                        .w_full()
                        .h(RESIZE_HANDLE_SIZE)
                        .cursor_row_resize(),
                ),
                DockPosition::Right => deferred(
                    handle
                        .absolute()
                        .top(px(0.))
                        .left(-RESIZE_HANDLE_SIZE / 2.)
                        .h_full()
                        .w(RESIZE_HANDLE_SIZE)
                        .cursor_col_resize(),
                ),
            }
        };

        div()
            .key_context("Dock")
            .track_focus(&self.focus_handle)
            .flex()
            .bg(cx.theme().panel)
            .border_color(cx.theme().border)
            .overflow_hidden()
            .map(|this| match self.position().axis() {
                Axis::Horizontal => this.w(size).h_full().flex_row(),
                Axis::Vertical => this.h(size).w_full().flex_col(),
            })
            .map(|this| match self.position() {
                DockPosition::Left => this.border_r_1(),
                DockPosition::Right => this.border_l_1(),
                DockPosition::Bottom => this.border_t_1(),
            })
            .child(
                div()
                    .map(|this| match self.position().axis() {
                        Axis::Horizontal => this.min_w(size).h_full(),
                        Axis::Vertical => this.min_h(size).w_full(),
                    })
                    .child(
                        entry
                            .panel
                            .to_any()
                            .cached(StyleRefinement::default().v_flex().size_full()),
                    ),
            )
            .when(self.resizeable, |this| this.child(create_resize_handle()))
    }
}
