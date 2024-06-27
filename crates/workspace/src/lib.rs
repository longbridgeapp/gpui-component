use gpui::*;
use prelude::FluentBuilder as _;

use std::sync::Arc;
use ui::{
    button::ButtonSize,
    input,
    label::Label,
    picker,
    switch::{LabelSide, Switch},
    theme::{ActiveTheme, Theme},
    title_bar::TitleBar,
};
use ui_story::Stories;
use util::ResultExt as _;

mod app_state;

pub use app_state::AppState;

pub struct Workspace {
    weak_self: WeakView<Self>,
    stories: View<Stories>,
}

impl Workspace {
    pub fn new(
        _app_state: Arc<AppState>,
        _parent: Option<WeakView<Self>>,
        cx: &mut ViewContext<Self>,
    ) -> Self {
        let weak_handle = cx.view().downgrade();

        Workspace {
            weak_self: weak_handle.clone(),
            stories: Stories::view(cx),
        }
    }

    pub fn new_local(
        app_state: Arc<AppState>,
        cx: &mut AppContext,
    ) -> Task<anyhow::Result<WindowHandle<Workspace>>> {
        ui::init(cx);

        let window_bounds = Bounds::centered(None, size(px(1200.0), px(900.0)), cx);

        cx.spawn(|mut cx| async move {
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(window_bounds)),
                titlebar: Some(TitlebarOptions {
                    title: None,
                    appears_transparent: true,
                    traffic_light_position: Some(point(px(9.0), px(9.0))),
                }),
                window_min_size: Size {
                    width: px(640.),
                    height: px(480.),
                },
                kind: WindowKind::Normal,
                ..Default::default()
            };

            let window = cx.open_window(options, |cx| {
                cx.new_view(|cx| Workspace::new(app_state.clone(), None, cx))
            })?;

            window
                .update(&mut cx, |_, cx| {
                    cx.activate_window();
                    cx.set_window_title("GPUI App");
                    cx.on_release(|_, _, _cx| {
                        // exit app
                        std::process::exit(0);
                    })
                    .detach();
                })
                .log_err();

            Ok(window)
        })
    }
}

actions!(workspace, [Open, CloseWindow]);

pub fn init(_app_state: Arc<AppState>, cx: &mut AppContext) {
    cx.on_action(|_action: &Open, _cx: &mut AppContext| {});
    cx.on_action(|_action: &CloseWindow, _cx| std::process::exit(0));

    Theme::init(cx);
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

impl Render for Workspace {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .relative()
            .flex()
            .flex_1()
            .flex_col()
            .size_full()
            .bg(theme.background)
            .gap_4()
            .child(
                TitleBar::new("main-title", Box::new(crate::CloseWindow))
                    .when(cfg!(not(windows)), |this| {
                        this.on_click(|event, cx| {
                            if event.up.click_count == 2 {
                                cx.zoom_window();
                            }
                        })
                    })
                    // left side
                    .child(div().flex().items_center().child(Label::new("GPUI App")))
                    .child(
                        div().flex().items_center().justify_end().px_2().child(
                            Switch::new("theme-mode")
                                .size(ButtonSize::Small)
                                .checked(cx.theme().mode.is_dark())
                                .label_side(LabelSide::Left)
                                .label("Dark Mode")
                                .on_click(cx.listener(|v, _, cx| {
                                    let mode = match cx.theme().mode {
                                        ui::theme::ThemeMode::Light => ui::theme::ThemeMode::Dark,
                                        ui::theme::ThemeMode::Dark => ui::theme::ThemeMode::Light,
                                    };

                                    Theme::change(mode, cx);
                                })),
                        ),
                    ),
            )
            .child(div().flex().px_4().gap_2().child(self.stories.clone()))
    }
}
