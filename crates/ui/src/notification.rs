use std::any::TypeId;
use std::borrow::Cow;
use std::sync::Arc;

use gpui::{
    AnyView, DismissEvent, ElementId, Entity, EntityId, EventEmitter, IntoElement, ParentElement,
    Render, SharedString, Styled, View, ViewContext, WindowContext,
};

use crate::theme::ActiveTheme;
use crate::{label::Label, v_flex, StyledExt};

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

// impl Workspace {
//     pub fn dismiss_notification(&mut self, id: &NotificationId, cx: &mut ViewContext<Self>) {
//         self.dismiss_notification_internal(id, cx)
//     }

//     pub fn show_toast(&mut self, toast: MessageNotification, cx: &mut ViewContext<Self>) {
//         self.dismiss_notification(&toast.id, cx);
//         self.show_notification(toast.id, cx, |cx| {
//             cx.new_view(|_cx| match toast.on_click.as_ref() {
//                 Some((click_msg, on_click)) => {
//                     let on_click = on_click.clone();
//                     simple_message_notification::MessageNotification::new(toast.msg.clone())
//                         .with_click_message(click_msg.clone())
//                         .on_click(move |cx| on_click(cx))
//                 }
//                 None => simple_message_notification::MessageNotification::new(toast.msg.clone()),
//             })
//         })
//     }
// }

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
    title: SharedString,
    description: SharedString,
}

impl MessageNotification {
    pub fn new(title: impl Into<SharedString>, description: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
        }
    }
}

impl Render for MessageNotification {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        v_flex()
            .elevation_3(cx)
            .p_4()
            .max_w_80()
            .bg(cx.theme().background)
            .child(
                v_flex()
                    .child(Label::new(self.title.clone()))
                    .child(Label::new(self.description.clone())),
            )
    }
}
