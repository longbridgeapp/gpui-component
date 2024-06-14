use gpui::{prelude::FluentBuilder, *};

use std::sync::Arc;
use ui::{
    button::{Button, ButtonStyle},
    disableable::Clickable as _,
    Color,
};
use util::ResultExt as _;

mod app_state;
mod item;

pub use app_state::AppState;

pub struct Workspace {
    weak_self: WeakView<Self>,
}

impl Workspace {
    pub fn new(
        app_state: Arc<AppState>,
        parent: Option<WeakView<Self>>,
        cx: &mut ViewContext<Self>,
    ) -> Self {
        let weak_handle = cx.view().downgrade();

        let workspace = Workspace {
            weak_self: weak_handle.clone(),
        };

        workspace
    }

    pub fn new_local(
        app_state: Arc<AppState>,
        cx: &mut AppContext,
    ) -> Task<anyhow::Result<WindowHandle<Workspace>>> {
        let window_bounds = Bounds::centered(None, size(px(1200.0), px(900.0)), cx);

        cx.spawn(|mut cx| async move {
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(window_bounds)),
                ..Default::default()
            };

            let window = cx.open_window(options, {
                let app_state = app_state.clone();
                move |cx| cx.new_view(|cx| Workspace::new(app_state.clone(), None, cx))
            })?;

            window
                .update(&mut cx, |_, cx| {
                    cx.activate_window();
                    cx.set_window_title("GPUI App");
                })
                .log_err();

            Ok(window)
        })
    }
}

actions!(workspace, [Open]);

pub fn init(app_state: Arc<AppState>, cx: &mut AppContext) {
    cx.on_action({
        let app_state = app_state.clone();
        move |action: &Open, cx: &mut AppContext| {}
    })
}

pub fn open_new(
    app_state: Arc<AppState>,
    cx: &mut AppContext,
    init: impl FnOnce(&mut Workspace, &mut ViewContext<Workspace>) + 'static + Send,
) -> Task<()> {
    let task = Workspace::new_local(app_state, cx);
    cx.spawn(|mut cx| async move {
        if let Some(workspace) = task.await.log_err() {
            workspace
                .update(&mut cx, |workspace, cx| init(workspace, cx))
                .log_err();
        }
    })
}

impl Workspace {
    pub fn render_ok_button(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        Button::new("ok-button", "OK")
            .style(ButtonStyle::Primary)
            .on_click(|_, cx| {
                // todo
            })
    }
}

impl Render for Workspace {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .relative()
            .flex()
            .flex_1()
            .flex_col()
            .size_full()
            .p_4()
            .bg(Color::Background.color(cx))
            .child(
                div()
                    .flex()
                    .py_3()
                    .gap_2()
                    .child(cx.new_view(|_| ui::story::Stories::new())),
            )
    }
}
