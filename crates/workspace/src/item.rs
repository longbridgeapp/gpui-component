use std::any::TypeId;

use gpui::{
    AnyElement, AnyView, AppContext, Element as _, Entity as _, EntityId, EventEmitter,
    FocusHandle, FocusableView, Pixels, Point, SharedString, View, ViewContext, WeakView,
    WindowContext,
};

use super::{
    pane::{self, Pane},
    workspace::{Workspace, WorkspaceId},
};

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum ItemEvent {
    CloseItem,
    UpdateTab,
    Edit,
}

#[derive(Debug, Clone, Copy)]
pub struct TabContentParams {
    pub detail: Option<usize>,
    pub selected: bool,
}

pub trait Item: FocusableView + EventEmitter<Self::Event> {
    type Event;

    /// Returns the content of the tab for this item.
    fn tab_content(&self, _params: TabContentParams, _cx: &WindowContext) -> AnyElement {
        gpui::Empty.into_any()
    }

    /// Returns the tooltip for the tab.
    fn tab_tooltip(&self, _: &AppContext) -> Option<SharedString> {
        None
    }

    /// Returns the description for the tab.
    fn tab_description(&self, _: usize, _: &AppContext) -> Option<SharedString> {
        None
    }

    fn to_item_events(_event: &Self::Event, _f: impl FnMut(ItemEvent)) {}

    /// Invoked when the item is deactivated.
    fn deactivated(&mut self, _: &mut ViewContext<Self>) {}

    /// Invoked when the workspace is deactivated.
    fn workspace_deactivated(&mut self, _cx: &mut ViewContext<Self>) {}

    fn is_singleton(&self, _cx: &AppContext) -> bool {
        false
    }

    fn clone_on_split(
        &self,
        _workspace_id: Option<WorkspaceId>,
        _: &mut ViewContext<Self>,
    ) -> Option<View<Self>>
    where
        Self: Sized,
    {
        None
    }

    fn act_as_type<'a>(
        &'a self,
        type_id: TypeId,
        self_handle: &'a View<Self>,
        _: &'a AppContext,
    ) -> Option<AnyView> {
        if TypeId::of::<Self>() == type_id {
            Some(self_handle.clone().into())
        } else {
            None
        }
    }

    fn added_to_workspace(&mut self, _workspace: &mut Workspace, _cx: &mut ViewContext<Self>) {}
    fn pixel_position_of_cursor(&self, _: &AppContext) -> Option<Point<Pixels>> {
        None
    }
}

pub trait ItemHandle: 'static + Send {
    fn item_id(&self) -> EntityId;
    fn subscribe_to_item_events(
        &self,
        cx: &mut WindowContext,
        handler: Box<dyn Fn(ItemEvent, &mut WindowContext)>,
    ) -> gpui::Subscription;
    fn focus_handle(&self, cx: &WindowContext) -> FocusHandle;
    fn tab_tooltip(&self, cx: &AppContext) -> Option<SharedString>;
    fn tab_description(&self, detail: usize, cx: &AppContext) -> Option<SharedString>;
    fn tab_content(&self, params: TabContentParams, cx: &WindowContext) -> AnyElement;
    fn dragged_tab_content(&self, params: TabContentParams, cx: &WindowContext) -> AnyElement;
    fn clone_on_split(
        &self,
        workspace_id: Option<WorkspaceId>,
        cx: &mut WindowContext,
    ) -> Option<Box<dyn ItemHandle>>;
    fn added_to_pane(
        &self,
        workspace: &mut Workspace,
        pane: View<Pane>,
        cx: &mut ViewContext<Workspace>,
    );
    fn deactivated(&self, cx: &mut WindowContext);
    fn workspace_deactivated(&self, cx: &mut WindowContext);
    fn to_any(&self) -> AnyView;
    fn on_release(
        &self,
        cx: &mut AppContext,
        callback: Box<dyn FnOnce(&mut AppContext) + Send>,
    ) -> gpui::Subscription;
    fn pixel_position_of_cursor(&self, cx: &AppContext) -> Option<Point<Pixels>>;
    fn downgrade_item(&self) -> Box<dyn WeakItemHandle>;
    fn boxed_clone(&self) -> Box<dyn ItemHandle>;
    fn act_as_type<'a>(&'a self, type_id: TypeId, cx: &'a AppContext) -> Option<AnyView>;
}

pub trait WeakItemHandle: Send + Sync {
    fn id(&self) -> EntityId;
    fn upgrade(&self) -> Option<Box<dyn ItemHandle>>;
}

impl dyn ItemHandle {
    pub fn downcast<V: 'static>(&self) -> Option<View<V>> {
        self.to_any().downcast().ok()
    }

    pub fn act_as<V: 'static>(&self, cx: &AppContext) -> Option<View<V>> {
        self.act_as_type(TypeId::of::<V>(), cx)
            .and_then(|t| t.downcast().ok())
    }
}

impl<T: Item> ItemHandle for View<T> {
    fn subscribe_to_item_events(
        &self,
        cx: &mut WindowContext,
        handler: Box<dyn Fn(ItemEvent, &mut WindowContext)>,
    ) -> gpui::Subscription {
        cx.subscribe(self, move |_, event, cx| {
            T::to_item_events(event, |item_event| handler(item_event, cx));
        })
    }

    fn focus_handle(&self, cx: &WindowContext) -> FocusHandle {
        self.focus_handle(cx)
    }

    fn tab_tooltip(&self, cx: &AppContext) -> Option<SharedString> {
        self.read(cx).tab_tooltip(cx)
    }

    fn tab_description(&self, detail: usize, cx: &AppContext) -> Option<SharedString> {
        self.read(cx).tab_description(detail, cx)
    }

    fn tab_content(&self, params: TabContentParams, cx: &WindowContext) -> AnyElement {
        self.read(cx).tab_content(params, cx)
    }

    fn dragged_tab_content(&self, params: TabContentParams, cx: &WindowContext) -> AnyElement {
        self.read(cx).tab_content(
            TabContentParams {
                selected: true,
                ..params
            },
            cx,
        )
    }

    fn boxed_clone(&self) -> Box<dyn ItemHandle> {
        Box::new(self.clone())
    }

    fn act_as_type<'a>(&'a self, type_id: TypeId, cx: &'a AppContext) -> Option<AnyView> {
        self.read(cx).act_as_type(type_id, self, cx)
    }

    fn clone_on_split(
        &self,
        workspace_id: Option<WorkspaceId>,
        cx: &mut WindowContext,
    ) -> Option<Box<dyn ItemHandle>> {
        self.update(cx, |item, cx| item.clone_on_split(workspace_id, cx))
            .map(|handle| Box::new(handle) as Box<dyn ItemHandle>)
    }

    fn added_to_pane(
        &self,
        workspace: &mut Workspace,
        pane: View<Pane>,
        cx: &mut ViewContext<Workspace>,
    ) {
        let _weak_item = self.downgrade();

        if workspace
            .panes_by_item
            .insert(self.item_id(), pane.downgrade())
            .is_none()
        {
            let mut event_subscription = Some(cx.subscribe(
                self,
                move |workspace, item: View<T>, event, cx| {
                    let pane = if let Some(pane) = workspace
                        .panes_by_item
                        .get(&item.item_id())
                        .and_then(|pane| pane.upgrade())
                    {
                        pane
                    } else {
                        return;
                    };

                    T::to_item_events(event, |event| match event {
                        ItemEvent::CloseItem => {
                            pane.update(cx, |pane, cx| pane.close_item_by_id(item.item_id(), cx))
                                .detach_and_log_err(cx);
                            return;
                        }

                        ItemEvent::UpdateTab => {
                            pane.update(cx, |_, cx| {
                                cx.emit(pane::Event::ChangeItemTitle);
                                cx.notify();
                            });
                        }

                        _ => {}
                    });
                },
            ));

            let item_id = self.item_id();
            cx.observe_release(self, move |workspace, _, _| {
                workspace.panes_by_item.remove(&item_id);
                event_subscription.take();
            })
            .detach();
        }

        // cx.defer(|workspace, cx| {
        //     workspace.serialize_workspace(cx);
        // });
    }

    fn deactivated(&self, cx: &mut WindowContext) {
        self.update(cx, |this, cx| this.deactivated(cx));
    }

    fn workspace_deactivated(&self, cx: &mut WindowContext) {
        self.update(cx, |this, cx| this.workspace_deactivated(cx));
    }

    fn item_id(&self) -> EntityId {
        self.entity_id()
    }

    fn to_any(&self) -> AnyView {
        self.clone().into()
    }

    fn on_release(
        &self,
        cx: &mut AppContext,
        callback: Box<dyn FnOnce(&mut AppContext) + Send>,
    ) -> gpui::Subscription {
        cx.observe_release(self, move |_, cx| callback(cx))
    }

    fn pixel_position_of_cursor(&self, cx: &AppContext) -> Option<Point<Pixels>> {
        self.read(cx).pixel_position_of_cursor(cx)
    }

    fn downgrade_item(&self) -> Box<dyn WeakItemHandle> {
        Box::new(self.downgrade())
    }
}

impl From<Box<dyn ItemHandle>> for AnyView {
    fn from(val: Box<dyn ItemHandle>) -> Self {
        val.to_any()
    }
}

impl From<&Box<dyn ItemHandle>> for AnyView {
    fn from(val: &Box<dyn ItemHandle>) -> Self {
        val.to_any()
    }
}

impl Clone for Box<dyn ItemHandle> {
    fn clone(&self) -> Box<dyn ItemHandle> {
        self.boxed_clone()
    }
}

impl<T: Item> WeakItemHandle for WeakView<T> {
    fn id(&self) -> EntityId {
        self.entity_id()
    }

    fn upgrade(&self) -> Option<Box<dyn ItemHandle>> {
        self.upgrade().map(|v| Box::new(v) as Box<dyn ItemHandle>)
    }
}
