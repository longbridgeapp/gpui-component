use gpui::{
    div, AnyView, FocusHandle, InteractiveElement, IntoElement, ParentElement as _, Render, Styled,
    View, ViewContext, VisualContext as _, WindowContext,
};
use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
};

use crate::{
    drawer::Drawer,
    modal::Modal,
    notification::{Notification, NotificationList},
    theme::ActiveTheme,
};

/// Extension trait for [`WindowContext`] and [`ViewContext`] to add drawer functionality.
pub trait ContextModal: Sized {
    /// Opens a Drawer.
    fn open_drawer<F>(&mut self, build: F)
    where
        F: Fn(Drawer, &mut WindowContext) -> Drawer + 'static;

    /// Return true, if there is an active Drawer.
    fn has_active_drawer(&self) -> bool;

    /// Closes the active Drawer.
    fn close_drawer(&mut self);

    /// Opens a Modal.
    fn open_modal<F>(&mut self, build: F)
    where
        F: Fn(Modal, &mut WindowContext) -> Modal + 'static;

    /// Return true, if there is an active Modal.
    fn has_active_modal(&self) -> bool;

    /// Closes the last active Modal.
    fn close_modal(&mut self);

    /// Closes all active Modals.
    fn close_all_modals(&mut self);

    /// Pushes a notification to the notification list.
    fn push_notification(&mut self, note: impl Into<Notification>);
    fn clear_notifications(&mut self);
    /// Returns number of notifications.
    fn notifications(&self) -> Rc<Vec<View<Notification>>>;
}

impl<'a> ContextModal for WindowContext<'a> {
    fn open_drawer<F>(&mut self, build: F)
    where
        F: Fn(Drawer, &mut WindowContext) -> Drawer + 'static,
    {
        Root::update(self, move |root, cx| {
            if root.active_drawer.is_none() {
                root.previous_focus_handle = cx.focused();
            }
            root.active_drawer = Some(Rc::new(build));
            cx.notify();
        })
    }

    fn has_active_drawer(&self) -> bool {
        Root::read(&self).active_drawer.is_some()
    }

    fn close_drawer(&mut self) {
        Root::update(self, |root, cx| {
            root.active_drawer = None;
            root.focus_back(cx);
            cx.notify();
        })
    }

    fn open_modal<F>(&mut self, build: F)
    where
        F: Fn(Modal, &mut WindowContext) -> Modal + 'static,
    {
        Root::update(self, move |root, cx| {
            // Only save focus handle if there are no active modals.
            // This is used to restore focus when all modals are closed.
            if root.active_modals.len() == 0 {
                root.previous_focus_handle = cx.focused();
            }

            let focus_handle = cx.focus_handle();
            focus_handle.focus(cx);

            root.active_modals.push(ActiveModal {
                focus_handle,
                builder: Rc::new(build),
            });
            cx.notify();
        })
    }

    fn has_active_modal(&self) -> bool {
        Root::read(&self).active_modals.len() > 0
    }

    fn close_modal(&mut self) {
        Root::update(self, move |root, cx| {
            root.active_modals.pop();

            if let Some(top_modal) = root.active_modals.last() {
                // Focus the next modal.
                top_modal.focus_handle.focus(cx);
            } else {
                // Restore focus if there are no more modals.
                root.focus_back(cx);
            }
            cx.notify();
        })
    }

    fn close_all_modals(&mut self) {
        Root::update(self, |root, cx| {
            root.active_modals.clear();
            root.focus_back(cx);
            cx.notify();
        })
    }

    fn push_notification(&mut self, note: impl Into<Notification>) {
        let note = note.into();
        Root::update(self, move |root, cx| {
            root.notification.update(cx, |view, cx| view.push(note, cx));
            cx.notify();
        })
    }

    fn clear_notifications(&mut self) {
        Root::update(self, move |root, cx| {
            root.notification.update(cx, |view, cx| view.clear(cx));
            cx.notify();
        })
    }

    fn notifications(&self) -> Rc<Vec<View<Notification>>> {
        Rc::new(Root::read(&self).notification.read(&self).notifications())
    }
}
impl<'a, V> ContextModal for ViewContext<'a, V> {
    fn open_drawer<F>(&mut self, build: F)
    where
        F: Fn(Drawer, &mut WindowContext) -> Drawer + 'static,
    {
        self.deref_mut().open_drawer(build)
    }

    fn has_active_modal(&self) -> bool {
        self.deref().has_active_modal()
    }

    fn close_drawer(&mut self) {
        self.deref_mut().close_drawer()
    }

    fn open_modal<F>(&mut self, build: F)
    where
        F: Fn(Modal, &mut WindowContext) -> Modal + 'static,
    {
        self.deref_mut().open_modal(build)
    }

    fn has_active_drawer(&self) -> bool {
        self.deref().has_active_drawer()
    }

    /// Close the last active modal.
    fn close_modal(&mut self) {
        self.deref_mut().close_modal()
    }

    /// Close all modals.
    fn close_all_modals(&mut self) {
        self.deref_mut().close_all_modals()
    }

    fn push_notification(&mut self, note: impl Into<Notification>) {
        self.deref_mut().push_notification(note)
    }

    fn clear_notifications(&mut self) {
        self.deref_mut().clear_notifications()
    }

    fn notifications(&self) -> Rc<Vec<View<Notification>>> {
        self.deref().notifications()
    }
}

/// Root is a view for the App window for as the top level view (Must be the first view in the window).
///
/// It is used to manage the Drawer, Modal, and Notification.
pub struct Root {
    /// Used to store the focus handle of the previus revious view.
    /// When the Modal, Drawer closes, we will focus back to the previous view.
    previous_focus_handle: Option<FocusHandle>,
    active_drawer: Option<Rc<dyn Fn(Drawer, &mut WindowContext) -> Drawer + 'static>>,
    active_modals: Vec<ActiveModal>,
    pub notification: View<NotificationList>,
    child: AnyView,
}

#[derive(Clone)]
struct ActiveModal {
    focus_handle: FocusHandle,
    builder: Rc<dyn Fn(Modal, &mut WindowContext) -> Modal + 'static>,
}

impl Root {
    pub fn new(child: AnyView, cx: &mut ViewContext<Self>) -> Self {
        Self {
            previous_focus_handle: None,
            active_drawer: None,
            active_modals: Vec::new(),
            notification: cx.new_view(NotificationList::new),
            child,
        }
    }

    pub fn update<F>(cx: &mut WindowContext, f: F)
    where
        F: FnOnce(&mut Self, &mut ViewContext<Self>) + 'static,
    {
        let root = cx
            .window_handle()
            .downcast::<Root>()
            .and_then(|w| w.root_view(cx).ok())
            .expect("The window root view should be of type `ui::Root`.");

        root.update(cx, |root, cx| f(root, cx))
    }

    pub fn read<'a>(cx: &'a WindowContext) -> &'a Self {
        let root = cx
            .window_handle()
            .downcast::<Root>()
            .and_then(|w| w.root_view(cx).ok())
            .expect("The window root view should be of type `ui::Root`.");

        root.read(cx)
    }

    fn focus_back(&mut self, cx: &mut WindowContext) {
        if let Some(handle) = self.previous_focus_handle.clone() {
            cx.focus(&handle);
        }
    }

    // Render Notification layer.
    pub fn render_notification_layer(cx: &mut WindowContext) -> Option<impl IntoElement> {
        let root = cx
            .window_handle()
            .downcast::<Root>()
            .and_then(|w| w.root_view(cx).ok())
            .expect("The window root view should be of type `ui::Root`.");

        Some(div().child(root.read(cx).notification.clone()))
    }

    /// Render the Drawer layer.
    pub fn render_drawer_layer(cx: &mut WindowContext) -> Option<impl IntoElement> {
        let root = cx
            .window_handle()
            .downcast::<Root>()
            .and_then(|w| w.root_view(cx).ok())
            .expect("The window root view should be of type `ui::Root`.");

        if let Some(builder) = root.read(cx).active_drawer.clone() {
            let drawer = Drawer::new(cx);
            return Some(builder(drawer, cx));
        }

        None
    }

    /// Render the Modal layer.
    pub fn render_modal_layer(cx: &mut WindowContext) -> Option<impl IntoElement> {
        let root = cx
            .window_handle()
            .downcast::<Root>()
            .and_then(|w| w.root_view(cx).ok())
            .expect("The window root view should be of type `ui::Root`.");

        let active_modals = root.read(cx).active_modals.clone();
        let mut has_overlay = false;

        if active_modals.is_empty() {
            return None;
        }

        Some(
            div().children(active_modals.iter().enumerate().map(|(i, active_modal)| {
                let mut modal = Modal::new(cx);

                modal = (active_modal.builder)(modal, cx);
                modal.layer_ix = i;
                // Give the modal the focus handle, because `modal` is a temporary value, is not possible to
                // keep the focus handle in the modal.
                //
                // So we keep the focus handle in the `active_modal`, this is owned by the `Root`.
                modal.focus_handle = active_modal.focus_handle.clone();

                // Keep only have one overlay, we only render the first modal with overlay.
                if has_overlay {
                    modal.overlay_visible = false;
                }
                if modal.has_overlay() {
                    has_overlay = true;
                }

                modal
            })),
        )
    }
}

impl Render for Root {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl IntoElement {
        div()
            .id("root")
            .size_full()
            .text_color(cx.theme().foreground)
            .child(self.child.clone())
    }
}
