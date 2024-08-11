use std::any::TypeId;
use std::borrow::Cow;
use std::sync::Arc;

use gpui::{
    div, AnyView, DismissEvent, ElementId, Entity, EntityId, EventEmitter, InteractiveElement,
    IntoElement, ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, View,
    ViewContext, VisualContext, WindowContext,
};

use ui::{h_flex, label::Label, theme::ActiveTheme, v_flex, Icon, IconName};

use crate::Workspace;

#[derive(Debug, PartialEq, Clone)]
pub struct NotificationId {
    /// A [`TypeId`] used to uniquely identify this notification.
    type_id: TypeId,
    /// A supplementary ID used to distinguish between multiple
    /// notifications that have the same [`type_id`](Self::type_id);
    id: Option<ElementId>,
}

impl NotificationId {
    /// Returns a unique [`NotificationId`] for the given type.
    pub fn unique<T: 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            id: None,
        }
    }

    /// Returns a [`NotificationId`] for the given type that is also identified
    /// by the provided ID.
    pub fn identified<T: 'static>(id: impl Into<ElementId>) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            id: Some(id.into()),
        }
    }
}

impl Workspace {
    pub fn show_notification<V: Notification>(
        &mut self,
        id: NotificationId,
        cx: &mut ViewContext<Self>,
        build_notification: impl FnOnce(&mut ViewContext<Self>) -> View<V>,
    ) {
        self.dismiss_notification_internal(&id, cx);

        let notification = build_notification(cx);
        cx.subscribe(&notification, {
            let id = id.clone();
            move |this, _, _: &DismissEvent, cx| {
                this.dismiss_notification_internal(&id, cx);
            }
        })
        .detach();
        self.notifications.push((id, Box::new(notification)));
        cx.notify();
    }

    pub fn dismiss_notification(&mut self, id: &NotificationId, cx: &mut ViewContext<Self>) {
        self.dismiss_notification_internal(id, cx)
    }

    fn dismiss_notification_internal(&mut self, id: &NotificationId, cx: &mut ViewContext<Self>) {
        self.notifications.retain(|(existing_id, _)| {
            if existing_id == id {
                cx.notify();
                false
            } else {
                true
            }
        });
    }

    pub fn show_toast(&mut self, toast: Toast, cx: &mut ViewContext<Self>) {
        self.dismiss_notification(&toast.id, cx);
        self.show_notification(toast.id, cx, |cx| {
            cx.new_view(|_cx| match toast.on_click.as_ref() {
                Some((click_msg, on_click)) => {
                    let on_click = on_click.clone();
                    MessageNotification::new(toast.msg.clone())
                        .with_click_message(click_msg.clone())
                        .on_click(move |cx| on_click(cx))
                }
                None => MessageNotification::new(toast.msg.clone()),
            })
        })
    }

    pub fn dismiss_toast(&mut self, id: &NotificationId, cx: &mut ViewContext<Self>) {
        self.dismiss_notification(id, cx);
    }

    pub fn clear_all_notifications(&mut self, cx: &mut ViewContext<Self>) {
        self.notifications.clear();
        cx.notify();
    }
}

pub struct Toast {
    id: NotificationId,
    msg: Cow<'static, str>,
    on_click: Option<(Cow<'static, str>, Arc<dyn Fn(&mut WindowContext)>)>,
}

impl Toast {
    pub fn new<I: Into<Cow<'static, str>>>(id: NotificationId, msg: I) -> Self {
        Toast {
            id,
            msg: msg.into(),
            on_click: None,
        }
    }

    pub fn on_click<F, M>(mut self, message: M, on_click: F) -> Self
    where
        M: Into<Cow<'static, str>>,
        F: Fn(&mut WindowContext) + 'static,
    {
        self.on_click = Some((message.into(), Arc::new(on_click)));
        self
    }
}

impl PartialEq for Toast {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.msg == other.msg
            && self.on_click.is_some() == other.on_click.is_some()
    }
}

impl Clone for Toast {
    fn clone(&self) -> Self {
        Toast {
            id: self.id.clone(),
            msg: self.msg.clone(),
            on_click: self.on_click.clone(),
        }
    }
}

pub trait Notification: EventEmitter<DismissEvent> + Render {}

impl<V: EventEmitter<DismissEvent> + Render> Notification for V {}

pub trait NotificationHandle: Send {
    fn id(&self) -> EntityId;
    fn to_any(&self) -> AnyView;
}

impl<T: Notification> NotificationHandle for View<T> {
    fn id(&self) -> EntityId {
        self.entity_id()
    }

    fn to_any(&self) -> AnyView {
        self.clone().into()
    }
}

impl From<&dyn NotificationHandle> for AnyView {
    fn from(val: &dyn NotificationHandle) -> Self {
        val.to_any()
    }
}

pub struct MessageNotification {
    message: SharedString,
    on_click: Option<Arc<dyn Fn(&mut ViewContext<Self>)>>,
    click_message: Option<SharedString>,
}

impl EventEmitter<DismissEvent> for MessageNotification {}

impl MessageNotification {
    pub fn new(message: impl Into<SharedString>) -> Self {
        Self {
            message: message.into(),
            on_click: None,
            click_message: None,
        }
    }

    pub fn with_click_message<S>(mut self, message: S) -> Self
    where
        S: Into<SharedString>,
    {
        self.click_message = Some(message.into());
        self
    }

    pub fn on_click<F>(mut self, on_click: F) -> Self
    where
        F: 'static + Fn(&mut ViewContext<Self>),
    {
        self.on_click = Some(Arc::new(on_click));
        self
    }

    pub fn dismiss(&mut self, cx: &mut ViewContext<Self>) {
        cx.emit(DismissEvent);
    }
}

impl Render for MessageNotification {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        v_flex()
            .bg(cx.theme().popover)
            .border_1()
            .border_color(cx.theme().border)
            .shadow_xl()
            .rounded_xl()
            .p_4()
            .max_w_80()
            .bg(cx.theme().background)
            .child(
                h_flex()
                    .justify_between()
                    .gap_2()
                    .child(div().max_w_80().child(Label::new(self.message.clone())))
                    .child(
                        div()
                            .id("cancel")
                            .child(Icon::new(IconName::Close))
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, cx| this.dismiss(cx))),
                    ),
            )
    }
}
