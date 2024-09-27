use gpui::{
    div, ClickEvent, FocusHandle, FocusableView, ParentElement as _, Render, Styled as _, View,
    ViewContext, VisualContext as _, WindowContext,
};
use ui::{
    button::Button,
    h_flex,
    input::{InputEvent, TextInput},
    theme::ActiveTheme,
    v_flex,
    webview::WebView,
    IconName,
};

pub struct WebViewStory {
    focus_handle: FocusHandle,
    webview: View<WebView>,
    address_input: View<TextInput>,
}

impl super::Story for WebViewStory {
    fn title() -> &'static str {
        "WebView"
    }

    fn new_view(cx: &mut WindowContext) -> View<impl gpui::FocusableView> {
        Self::view(cx)
    }
}

impl WebViewStory {
    pub fn view(cx: &mut WindowContext) -> View<Self> {
        let focus_handle = cx.focus_handle();

        let webview = cx.new_view(|cx| {
            let webview = ui::wry::WebViewBuilder::new_as_child(&cx.raw_window_handle())
                .build()
                .unwrap();
            WebView::new(cx, webview)
        });

        let address_input = cx.new_view(|cx| {
            let mut input = TextInput::new(cx);
            input.set_text("https://github.com/explore", cx);
            input
        });

        let url = address_input.read(cx).text();
        webview.update(cx, |view, _| {
            view.load_url(&url);
        });

        cx.new_view(|cx| {
            let this = WebViewStory {
                focus_handle,
                webview,
                address_input: address_input.clone(),
            };

            cx.subscribe(
                &address_input,
                |this: &mut Self, input, event: &InputEvent, cx| match event {
                    InputEvent::PressEnter => {
                        let url = input.read(cx).text();
                        this.webview.update(cx, |view, _| {
                            view.load_url(&url);
                        });
                    }
                    _ => {}
                },
            )
            .detach();

            this
        })
    }

    pub fn hide(&self, cx: &mut WindowContext) {
        self.webview.update(cx, |webview, _| webview.hide())
    }

    fn go_back(&mut self, _: &ClickEvent, cx: &mut ViewContext<Self>) {
        self.webview.update(cx, |webview, _| {
            webview.back().unwrap();
        });
    }
}

impl FocusableView for WebViewStory {
    fn focus_handle(&self, _cx: &gpui::AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for WebViewStory {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
        v_flex()
            .p_2()
            .gap_3()
            .size_full()
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        Button::new("go-back")
                            .icon(IconName::ArrowLeft)
                            .on_click(cx.listener(Self::go_back)),
                    )
                    .child(self.address_input.clone()),
            )
            .child(
                div()
                    .size_full()
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(self.webview.clone()),
            )
    }
}
