use std::rc::Rc;

use wry::{
    dpi::{self, LogicalPosition, LogicalSize},
    Rect,
};

use gpui::{
    div, AppContext, Bounds, ContentMask, DismissEvent, Element, ElementId, EventEmitter,
    FocusHandle, FocusableView, GlobalElementId, Hitbox, InteractiveElement, IntoElement, LayoutId,
    MouseDownEvent, MouseEvent, ParentElement as _, Pixels, Point, Render, Size, Style,
    Styled as _, View, WindowContext,
};

use crate::event;

pub fn init(_cx: &AppContext) {}

pub struct WebView {
    focus_handle: FocusHandle,
    webview: Rc<wry::WebView>,
    visable: bool,
}

impl WebView {
    pub fn new(cx: &mut WindowContext) -> Self {
        let focus_handle = cx.focus_handle();
        let window_handle = cx.raw_window_handle();

        let webview = wry::WebView::new_as_child(&window_handle)
            .expect("failed to create webview to child window");

        Self {
            focus_handle,
            visable: true,
            webview,
        }
    }

    pub fn show(&mut self) {
        let _ = self.webview.set_visible(true);
    }

    pub fn hide(&mut self) {
        let _ = self.webview.set_visible(false);
    }

    pub fn visible(&self) -> bool {
        self.visable
    }

    pub fn load_url(&mut self, url: &str) {
        self.webview.load_url(url).unwrap();
    }
}

impl FocusableView for WebView {
    fn focus_handle(&self, _cx: &gpui::AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<DismissEvent> for WebView {}

impl Render for WebView {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl IntoElement {
        let view = cx.view().clone();

        div()
            .track_focus(&self.focus_handle)
            .size_full()
            .debug()
            .child(WebViewElement::new(self.webview.clone(), view, cx))
    }
}

/// A webview element can display a wry webview.
pub struct WebViewElement {
    parent: View<WebView>,
    view: Rc<wry::WebView>,
}

impl WebViewElement {
    /// Create a new webview element from a wry WebView.
    pub fn new(view: Rc<wry::WebView>, parent: View<WebView>, _: &mut WindowContext) -> Self {
        Self { view, parent }
    }
}

impl IntoElement for WebViewElement {
    type Element = WebViewElement;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for WebViewElement {
    type RequestLayoutState = ();
    type PrepaintState = Option<Hitbox>;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        cx: &mut WindowContext,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.flex_grow = 0.0;
        style.flex_shrink = 1.;
        style.size = Size::full();
        // If the parent view is no longer visible, we don't need to layout the webview

        let id = cx.request_layout(style, []);
        (id, ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        cx: &mut WindowContext,
    ) -> Self::PrepaintState {
        if !self.parent.read(cx).visible() {
            return None;
        }

        if bounds.top() > cx.viewport_size().height || bounds.bottom() < Pixels::ZERO {
            // self.view.set_visible(false).unwrap();
        } else {
            // self.view.set_visible(true).unwrap();

            self.view
                .set_bounds(Rect {
                    size: dpi::Size::Logical(LogicalSize {
                        width: (bounds.size.width.0).into(),
                        height: (bounds.size.height.0).into(),
                    }),
                    position: dpi::Position::Logical(dpi::LogicalPosition::new(
                        bounds.origin.x.into(),
                        bounds.origin.y.into(),
                    )),
                })
                .unwrap();
        };

        // Create a hitbox to handle mouse event
        Some(cx.insert_hitbox(bounds, false))
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        _: &mut Self::PrepaintState,
        _: &mut WindowContext,
    ) {
        let bounds = hitbox.clone().map(|h| h.bounds).unwrap_or(bounds);
        cx.with_content_mask(Some(ContentMask { bounds }), |cx| {
            let webview = self.view.clone();
            cx.on_mouse_event(move |event: &MouseDownEvent, _, cx| {
                if !bounds.contains(&event.position) {
                    println!("Click outside the WebView.");
                    // Click white space to blur the input focus
                    webview
                        .evaluate_script(
                            r#"
                        document.querySelectorAll("input").forEach(input => input.blur());
                        "#,
                        )
                        .expect("Failed to click");
                } else {
                    println!("Click inside the WebView.");
                    cx.blur();
                }
            });
        });
    }
}
