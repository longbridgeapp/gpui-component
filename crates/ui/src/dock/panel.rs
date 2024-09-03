use anyhow::Result;
use gpui::{
    AnyView, EventEmitter, FocusableView, SharedString, Task, View, ViewContext, WindowContext,
};
use rust_i18n::t;

use super::{DockArea, PanelEvent, PanelId, TabPanel};
use crate::popup_menu::PopupMenu;

pub trait Panel: EventEmitter<PanelEvent> + FocusableView {
    /// The name of the panel, used to save/load the layout.
    ///
    /// For example: ResetPassword, Feedback, etc.
    fn panel_name() -> &'static str;

    /// Panel id used to identify the panel, this should be unique in entire application.
    ///
    /// It used to save/load the layout.
    fn panel_id(&self) -> PanelId;

    /// The title of the panel, default is `None`.
    fn title(&self, _cx: &WindowContext) -> SharedString {
        t!("Dock.Unnamed").into()
    }

    /// Whether the panel can be closed, default is `true`.
    fn closeable(&self, _cx: &WindowContext) -> bool {
        true
    }

    /// The addition popup menu of the panel, default is `None`.
    fn popup_menu(&self, this: PopupMenu, _cx: &WindowContext) -> PopupMenu {
        this
    }

    fn deserialize(
        _doc_area: View<DockArea>,
        _panel_id: PanelId,
        _cx: &mut ViewContext<TabPanel>,
    ) -> Task<Result<Box<dyn PanelView>>> {
        Task::Ready(None)
    }

    fn serialize(
        &self,
        _dock_area: View<DockArea>,
        _cx: &mut ViewContext<TabPanel>,
    ) -> Task<Result<()>> {
        Task::Ready(None)
    }
}

pub trait PanelView: 'static + Send + Sync {
    fn panel_name(&self) -> &'static str;

    /// Item id used to identify the panel, this should be unique in entire application.
    ///
    /// It used to save/load the layout.
    fn panel_id(&self, cx: &WindowContext) -> PanelId;

    /// The title of the panel, default is `None`.
    fn title(&self, _cx: &WindowContext) -> SharedString {
        t!("Dock.Unnamed").into()
    }

    fn closeable(&self, cx: &WindowContext) -> bool;

    fn popup_menu(&self, menu: PopupMenu, cx: &WindowContext) -> PopupMenu;

    fn view(&self) -> AnyView;
}

impl<T: Panel> PanelView for View<T> {
    fn panel_name(&self) -> &'static str {
        T::panel_name()
    }

    fn panel_id(&self, cx: &WindowContext) -> PanelId {
        self.read(cx).panel_id()
    }

    fn title(&self, cx: &WindowContext) -> SharedString {
        self.read(cx).title(cx)
    }

    fn closeable(&self, cx: &WindowContext) -> bool {
        self.read(cx).closeable(cx)
    }

    fn popup_menu(&self, menu: PopupMenu, cx: &WindowContext) -> PopupMenu {
        self.read(cx).popup_menu(menu, cx)
    }

    fn view(&self) -> AnyView {
        self.clone().into()
    }
}

impl From<&dyn PanelView> for AnyView {
    fn from(handle: &dyn PanelView) -> Self {
        handle.view()
    }
}

impl<T: Panel> From<&dyn PanelView> for View<T> {
    fn from(value: &dyn PanelView) -> Self {
        value.view().downcast::<T>().unwrap()
    }
}

impl PartialEq for dyn PanelView {
    fn eq(&self, other: &Self) -> bool {
        self.view() == other.view()
    }
}
