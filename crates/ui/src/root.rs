use gpui::{
    div, prelude::FluentBuilder as _, AnyView, FocusHandle, InteractiveElement, ParentElement as _,
    Render, Styled, ViewContext, WindowContext,
};
use std::{ops::DerefMut, rc::Rc};

use crate::{drawer::Drawer, modal::Modal, theme::ActiveTheme, StyledExt};

/// Extension trait for [`WindowContext`] and [`ViewContext`] to add drawer functionality.
pub trait ContextModal: Sized {
    /// Opens a Drawer.
    fn open_drawer<F>(&mut self, build: F)
    where
        F: Fn(Drawer, &mut WindowContext) -> Drawer + 'static;

    /// Closes the active Drawer.
    fn close_drawer(&mut self);

    /// Opens a Modal.
    fn open_modal<F>(&mut self, build: F)
    where
        F: Fn(Modal, &mut WindowContext) -> Modal + 'static;

    /// Closes the active Modal.
    fn close_modal(&mut self);
}

impl<'a> ContextModal for WindowContext<'a> {
    fn open_drawer<F>(&mut self, build: F)
    where
        F: Fn(Drawer, &mut WindowContext) -> Drawer + 'static,
    {
        Root::update_root(self, move |root, cx| {
            root.active_drawer = Some(Rc::new(build));
            cx.notify();
        })
    }

    fn close_drawer(&mut self) {
        Root::update_root(self, |root, cx| {
            root.active_drawer = None;
            cx.notify();
        })
    }

    fn open_modal<F>(&mut self, build: F)
    where
        F: Fn(Modal, &mut WindowContext) -> Modal + 'static,
    {
        Root::update_root(self, move |root, cx| {
            root.active_modal = Some(Rc::new(build));
            cx.notify();
        })
    }

    fn close_modal(&mut self) {
        Root::update_root(self, |root, cx| {
            root.active_modal = None;
            cx.notify();
        })
    }
}
impl<'a, V> ContextModal for ViewContext<'a, V> {
    fn open_drawer<F>(&mut self, build: F)
    where
        F: Fn(Drawer, &mut WindowContext) -> Drawer + 'static,
    {
        self.deref_mut().open_drawer(build)
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

    fn close_modal(&mut self) {
        self.deref_mut().close_modal()
    }
}

pub struct Root {
    focus_handle: FocusHandle,
    active_drawer: Option<Rc<dyn Fn(Drawer, &mut WindowContext) -> Drawer + 'static>>,
    active_modal: Option<Rc<dyn Fn(Modal, &mut WindowContext) -> Modal + 'static>>,
    root_view: AnyView,
}

impl Root {
    pub fn new(root_view: AnyView, cx: &mut ViewContext<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            active_drawer: None,
            active_modal: None,
            root_view,
        }
    }

    fn update_root<F>(cx: &mut WindowContext, f: F)
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
}

impl Render for Root {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
        let has_modal = self.active_modal.is_some();

        div()
            .track_focus(&self.focus_handle)
            .size_full()
            .debug_focused(&self.focus_handle, cx)
            .text_color(cx.theme().foreground)
            .child(self.root_view.clone())
            .when(!has_modal, |this| {
                this.when_some(self.active_drawer.clone(), |this, build| {
                    let drawer = Drawer::new(cx);
                    this.child(build(drawer, cx))
                })
            })
            .when_some(self.active_modal.clone(), |this, build| {
                let modal = Modal::new(cx);
                this.child(build(modal, cx))
            })
    }
}
