use crate::{theme::ActiveTheme, Selectable, StyledExt as _};
use gpui::{
    actions, deferred, div, AnchorCorner, AnyElement, Element, ElementId, InteractiveElement,
    IntoElement, LayoutId, ParentElement as _, Render, StatefulInteractiveElement, Styled as _,
    View, ViewContext, VisualContext, WindowContext,
};

actions!(popover, [Dismiss]);

pub struct PopoverContent {
    content: Box<dyn Fn(&WindowContext) -> AnyElement>,
}

impl PopoverContent {
    pub fn new<C>(content: C) -> Self
    where
        C: Fn(&WindowContext) -> AnyElement + 'static,
    {
        Self {
            content: Box::new(content),
        }
    }
}

impl Render for PopoverContent {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div().child((self.content)(cx))
    }
}

pub struct Popover {
    id: ElementId,
    anchor: AnchorCorner,
    trigger: Option<Box<dyn FnOnce(&WindowContext) -> AnyElement + 'static>>,
    content: Option<View<PopoverContent>>,
    open: bool,
}

impl Popover {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            anchor: AnchorCorner::TopLeft,
            trigger: None,
            content: None,
            open: false,
        }
    }

    pub fn anchor(mut self, anchor: AnchorCorner) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn trigger<T>(mut self, trigger: T) -> Self
    where
        T: Selectable + IntoElement + 'static,
    {
        self.trigger = Some(Box::new(|_| trigger.into_any_element()));
        self
    }

    /// Set the content of the popover.
    ///
    /// The `content` is a closure that returns an `AnyElement`.
    pub fn content<C>(mut self, content: C, cx: &mut WindowContext) -> Self
    where
        C: Fn(&WindowContext) -> AnyElement + 'static,
    {
        let view = cx.new_view(|_| PopoverContent::new(content));
        self.content = Some(view);
        self
    }

    fn render_trigger(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let base = div().id("popover-trigger");

        if self.trigger.is_none() {
            return base;
        }

        let trigger = self.trigger.take().unwrap();

        base.child((trigger)(cx))
            .on_click(cx.listener(|this, _, cx| {
                dbg!("--------- click");
                this.open = !this.open;
                cx.notify();
            }))
            .into_element()
    }

    fn render_popover(&self, cx: &WindowContext) -> impl IntoElement {
        let base = div().id("popover-content");

        if self.content.is_none() {
            return base;
        }

        let content = self.content.as_ref().unwrap();

        base.absolute()
            .mt_2()
            .elevation_2(cx)
            .bg(cx.theme().popover)
            .border_1()
            .border_color(cx.theme().border)
            .p_4()
            .max_w_128()
            .w_80()
            .occlude()
            .child(content.clone())
            .on_mouse_down_out(|_, cx| {
                cx.dispatch_action(Box::new(Dismiss));
            })
            .into_element()
    }
}

impl Render for Popover {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .key_context("Popover")
            .id(self.id.clone())
            .on_click(cx.listener(|this, _, cx| {
                dbg!("--------- click");
                this.open = !this.open;
                cx.notify();
            }))
            .child(self.render_trigger(cx))
            .child(PopoverElement {
                id: self.id.clone(),
                popover: cx.view().clone(),
            })
    }
}

struct PopoverElement {
    id: ElementId,
    popover: View<Popover>,
}

impl IntoElement for PopoverElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

struct PopoverElementState {
    trigger_bounds: Option<gpui::Bounds<gpui::Pixels>>,
    layout_id: LayoutId,
    popover_element: AnyElement,
}

impl Element for PopoverElement {
    type RequestLayoutState = PopoverElementState;

    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn request_layout(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        cx: &mut WindowContext,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        let popover_element = self.popover.read(cx).render_popover(cx).into_any_element();

        let mut popover_element = deferred(popover_element).with_priority(1).into_any();
        let layout_id = popover_element.request_layout(cx);

        (
            layout_id.clone(),
            PopoverElementState {
                trigger_bounds: None,
                layout_id,
                popover_element,
            },
        )
    }

    fn prepaint(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        cx: &mut WindowContext,
    ) -> Self::PrepaintState {
        if self.popover.read(cx).open {
            request_layout.popover_element.prepaint(cx);
        }
    }

    fn paint(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        cx: &mut WindowContext,
    ) {
        if self.popover.read(cx).open {
            request_layout.popover_element.paint(cx);
        }
    }
}
