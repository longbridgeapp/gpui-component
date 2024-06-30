use std::collections::{hash_map, HashMap};

use gpui::{
    AnyWeakView, Entity as _, EntityId, EventEmitter, View, ViewContext, VisualContext as _,
    WeakView,
};

use super::{
    dock::{Dock, DockPosition},
    pane::{self, Pane},
    pane_group::{PaneGroup, SplitDirection},
};

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct WorkspaceId(i64);

/// Workspace is a container for docks.
pub struct Workspace {
    weak_self: WeakView<Self>,
    center: PaneGroup,
    left_dock: View<Dock>,
    bottom_dock: View<Dock>,
    right_dock: View<Dock>,
    panes: Vec<View<Pane>>,
    pub(crate) panes_by_item: HashMap<EntityId, WeakView<Pane>>,
    active_pane: View<Pane>,
    last_active_center_pane: Option<WeakView<Pane>>,
    zoomed: Option<AnyWeakView>,
    zoomed_position: Option<DockPosition>,
    database_id: Option<WorkspaceId>,
}

pub enum Event {
    PaneAdded(View<Pane>),
    PaneRemoved,
    ItemAdded,
    ItemRemoved,
    ActiveItemChanged,
    WorkspaceCreated(WeakView<Workspace>),
    ZoomChanged,
}

impl EventEmitter<Event> for Workspace {}

impl Workspace {
    pub fn weak_handle(&self) -> WeakView<Self> {
        self.weak_self.clone()
    }

    pub fn left_dock(&self) -> &View<Dock> {
        &self.left_dock
    }

    pub fn bottom_dock(&self) -> &View<Dock> {
        &self.bottom_dock
    }

    pub fn right_dock(&self) -> &View<Dock> {
        &self.right_dock
    }

    pub fn database_id(&self) -> Option<WorkspaceId> {
        self.database_id
    }

    fn add_pane(&mut self, cx: &mut ViewContext<Self>) -> View<Pane> {
        let pane = cx.new_view(|cx| Pane::new(self.weak_handle(), None, cx));
        cx.subscribe(&pane, Self::handle_pane_event).detach();
        self.panes.push(pane.clone());
        cx.focus_view(&pane);
        cx.emit(Event::PaneAdded(pane.clone()));
        pane
    }

    pub fn split_pane(
        &mut self,
        pane_to_split: View<Pane>,
        split_direction: SplitDirection,
        cx: &mut ViewContext<Self>,
    ) -> View<Pane> {
        let new_pane = self.add_pane(cx);
        self.center
            .split(&pane_to_split, &new_pane, split_direction)
            .unwrap();
        cx.notify();
        new_pane
    }

    pub fn split_and_clone(
        &mut self,
        pane: View<Pane>,
        direction: SplitDirection,
        cx: &mut ViewContext<Self>,
    ) -> Option<View<Pane>> {
        let item = pane.read(cx).active_item()?;
        let maybe_pane_handle = if let Some(clone) = item.clone_on_split(self.database_id(), cx) {
            let new_pane = self.add_pane(cx);
            new_pane.update(cx, |pane, cx| pane.add_item(clone, true, true, None, cx));
            self.center.split(&pane, &new_pane, direction).unwrap();
            Some(new_pane)
        } else {
            None
        };
        cx.notify();
        maybe_pane_handle
    }

    pub fn split_pane_with_item(
        &mut self,
        pane_to_split: WeakView<Pane>,
        split_direction: SplitDirection,
        from: WeakView<Pane>,
        item_id_to_move: EntityId,
        cx: &mut ViewContext<Self>,
    ) {
        let Some(pane_to_split) = pane_to_split.upgrade() else {
            return;
        };
        let Some(from) = from.upgrade() else {
            return;
        };

        let new_pane = self.add_pane(cx);
        self.move_item(from.clone(), new_pane.clone(), item_id_to_move, 0, cx);
        self.center
            .split(&pane_to_split, &new_pane, split_direction)
            .unwrap();
        cx.notify();
    }

    pub fn move_item(
        &mut self,
        source: View<Pane>,
        destination: View<Pane>,
        item_id_to_move: EntityId,
        destination_index: usize,
        cx: &mut ViewContext<Self>,
    ) {
        let Some((item_ix, item_handle)) = source
            .read(cx)
            .items()
            .enumerate()
            .find(|(_, item_handle)| item_handle.item_id() == item_id_to_move)
        else {
            // Tab was closed during drag
            return;
        };

        let item_handle = item_handle.clone();

        if source != destination {
            // Close item from previous pane
            source.update(cx, |source, cx| {
                source.remove_item(item_ix, false, cx);
            });
        }

        // This automatically removes duplicate items in the pane
        destination.update(cx, |destination, cx| {
            destination.add_item(item_handle, true, true, Some(destination_index), cx);
            destination.focus(cx)
        });
    }

    fn remove_pane(&mut self, pane: View<Pane>, cx: &mut ViewContext<Self>) {
        if self.center.remove(&pane).unwrap() {
            self.force_remove_pane(&pane, cx);

            for removed_item in pane.read(cx).items() {
                self.panes_by_item.remove(&removed_item.item_id());
            }

            cx.notify();
        }

        cx.emit(Event::PaneRemoved);
    }

    fn force_remove_pane(&mut self, pane: &View<Pane>, cx: &mut ViewContext<Workspace>) {
        self.panes.retain(|p| p != pane);
        self.panes
            .last()
            .unwrap()
            .update(cx, |pane, cx| pane.focus(cx));
        if self.last_active_center_pane == Some(pane.downgrade()) {
            self.last_active_center_pane = None;
        }
        cx.notify();
    }

    pub fn panes(&self) -> &[View<Pane>] {
        &self.panes
    }

    pub fn active_pane(&self) -> &View<Pane> {
        &self.active_pane
    }

    fn handle_pane_event(
        &mut self,
        pane: View<Pane>,
        event: &pane::Event,
        cx: &mut ViewContext<Self>,
    ) {
        match event {
            pane::Event::AddItem { item } => {
                item.added_to_pane(self, pane, cx);
                cx.emit(Event::ItemAdded);
            }
            pane::Event::Split(direction) => {
                self.split_and_clone(pane, *direction, cx);
            }
            pane::Event::Remove => self.remove_pane(pane, cx),
            pane::Event::ActivateItem { local } => {
                cx.emit(Event::ActiveItemChanged);
            }
            pane::Event::ChangeItemTitle => {
                cx.emit(Event::ActiveItemChanged);
            }
            pane::Event::RemoveItem { item_id } => {
                cx.emit(Event::ActiveItemChanged);

                if let hash_map::Entry::Occupied(entry) = self.panes_by_item.entry(*item_id) {
                    if entry.get().entity_id() == pane.entity_id() {
                        entry.remove();
                    }
                }
            }
            pane::Event::Focus => {
                self.handle_pane_focused(pane.clone(), cx);
            }
            pane::Event::ZoomIn => {
                if pane == self.active_pane {
                    pane.update(cx, |pane, cx| pane.set_zoomed(true, cx));
                    if pane.read(cx).has_focus(cx) {
                        self.zoomed = Some(pane.downgrade().into());
                        self.zoomed_position = None;
                        cx.emit(Event::ZoomChanged);
                    }
                    cx.notify();
                }
            }
            pane::Event::ZoomOut => {
                pane.update(cx, |pane, cx| pane.set_zoomed(false, cx));
                if self.zoomed_position.is_none() {
                    self.zoomed = None;
                    cx.emit(Event::ZoomChanged);
                }
                cx.notify();
            }
        }

        self.serialize_workspace(cx);
    }

    fn handle_pane_focused(&mut self, pane: View<Pane>, cx: &mut ViewContext<Self>) {
        if self.active_pane != pane {
            self.active_pane = pane.clone();
            self.last_active_center_pane = Some(pane.downgrade());
        }

        self.dismiss_zoomed_items_to_reveal(None, cx);
        if pane.read(cx).is_zoomed() {
            self.zoomed = Some(pane.downgrade().into());
        } else {
            self.zoomed = None;
        }
        self.zoomed_position = None;
        cx.emit(Event::ZoomChanged);

        cx.notify();
    }

    fn dismiss_zoomed_items_to_reveal(
        &mut self,
        dock_to_reveal: Option<DockPosition>,
        cx: &mut ViewContext<Self>,
    ) {
        // If a center pane is zoomed, unzoom it.
        for pane in &self.panes {
            if pane != &self.active_pane || dock_to_reveal.is_some() {
                pane.update(cx, |pane, cx| pane.set_zoomed(false, cx));
            }
        }

        // If another dock is zoomed, hide it.
        let mut focus_center = false;
        for dock in [&self.left_dock, &self.right_dock, &self.bottom_dock] {
            dock.update(cx, |dock, cx| {
                if Some(dock.position()) != dock_to_reveal {
                    if let Some(panel) = dock.active_panel() {
                        if panel.is_zoomed(cx) {
                            focus_center |= panel.focus_handle(cx).contains_focused(cx);
                            dock.set_open(false, cx);
                        }
                    }
                }
            });
        }

        if focus_center {
            self.active_pane.update(cx, |pane, cx| pane.focus(cx))
        }

        if self.zoomed_position != dock_to_reveal {
            self.zoomed = None;
            self.zoomed_position = None;
            cx.emit(Event::ZoomChanged);
        }

        cx.notify();
    }

    pub(crate) fn serialize_workspace(&mut self, cx: &mut ViewContext<Self>) {
        // if self._schedule_serialize.is_none() {
        //     self._schedule_serialize = Some(cx.spawn(|this, mut cx| async move {
        //         cx.background_executor()
        //             .timer(Duration::from_millis(100))
        //             .await;
        //         this.update(&mut cx, |this, cx| {
        //             this.serialize_workspace_internal(cx).detach();
        //             this._schedule_serialize.take();
        //         })
        //         .log_err();
        //     }));
        // }
    }
}
