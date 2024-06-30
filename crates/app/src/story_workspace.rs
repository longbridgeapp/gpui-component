use gpui::*;
use prelude::FluentBuilder as _;
use workspace::{
    dock::{DockPosition, Panel, PanelEvent},
    notification::{NotificationHandle, NotificationId},
    Workspace,
};

use std::sync::Arc;
use ui::{
    button::ButtonSize,
    switch::{LabelSide, Switch},
    theme::{ActiveTheme, Theme},
    title_bar::TitleBar,
};
use ui_story::Stories;
use util::ResultExt as _;

use crate::app_state::AppState;

actions!(workspace, [Open, CloseWindow]);

pub fn init(_app_state: Arc<AppState>, cx: &mut AppContext) {
    cx.on_action(|_action: &Open, _cx: &mut AppContext| {});
    cx.on_action(|_action: &CloseWindow, _cx| std::process::exit(0));

    Theme::init(cx);
    ui::init(cx);
}

struct StoriesPanel {
    focus_handle: FocusHandle,
    stories: View<Stories>,
    position: DockPosition,
    width: Option<Pixels>,
}

impl StoriesPanel {
    fn new(cx: &mut WindowContext) -> View<Self> {
        let focus_handle = cx.focus_handle();
        let stories = Stories::view(cx);

        cx.new_view(|_| Self {
            focus_handle,
            stories,
            position: DockPosition::Left,
            width: None,
        })
    }
}

impl Panel for StoriesPanel {
    fn persistent_name() -> &'static str {
        "stories-panel"
    }

    fn position(&self, cx: &WindowContext) -> workspace::dock::DockPosition {
        self.position
    }

    fn can_position(&self, position: workspace::dock::DockPosition, cx: &WindowContext) -> bool {
        true
    }

    fn size(&self, cx: &WindowContext) -> Pixels {
        px(680.)
    }

    fn set_position(&mut self, position: workspace::dock::DockPosition, cx: &mut WindowContext) {
        self.position = position;
    }

    fn set_size(&mut self, size: Option<Pixels>, cx: &mut WindowContext) {
        if let Some(size) = size {
            self.width = Some(size);
        }
    }

    fn set_active(&mut self, active: bool, cx: &mut WindowContext) {}

    fn starts_open(&self, _cx: &WindowContext) -> bool {
        true
    }
}

impl FocusableView for StoriesPanel {
    fn focus_handle(&self, cx: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<PanelEvent> for StoriesPanel {}

impl Render for StoriesPanel {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        self.stories.clone()
    }
}

pub struct StoryWorkspace {
    workspace: View<Workspace>,
}

impl StoryWorkspace {
    pub fn new(
        _app_state: Arc<AppState>,
        workspace: View<Workspace>,
        cx: &mut ViewContext<Self>,
    ) -> Self {
        cx.observe_window_appearance(|_workspace, cx| {
            Theme::sync_system_appearance(cx);
        })
        .detach();

        let panel = StoriesPanel::new(cx);
        workspace.update(cx, |workspace, cx| {
            workspace.add_panel(panel, cx);
        });

        Self { workspace }
    }

    pub fn new_local(
        app_state: Arc<AppState>,
        cx: &mut AppContext,
    ) -> Task<anyhow::Result<WindowHandle<Self>>> {
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
                let workspace = cx.new_view(|cx| Workspace::new(None, cx));
                cx.new_view(|cx| Self::new(app_state.clone(), workspace, cx))
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

pub fn open_new(
    app_state: Arc<AppState>,
    cx: &mut AppContext,
    init: impl FnOnce(&mut StoryWorkspace, &mut ViewContext<StoryWorkspace>) + 'static + Send,
) -> Task<()> {
    let task: Task<std::result::Result<WindowHandle<StoryWorkspace>, anyhow::Error>> =
        StoryWorkspace::new_local(app_state, cx);
    cx.spawn(|mut cx| async move {
        if let Some(workspace) = task.await.log_err() {
            workspace
                .update(&mut cx, |workspace, cx| init(workspace, cx))
                .log_err();
        }
    })
}

impl Render for StoryWorkspace {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .relative()
            .size_full()
            .flex()
            .flex_1()
            .flex_col()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(
                TitleBar::new("main-title", Box::new(CloseWindow))
                    .when(cfg!(not(windows)), |this| {
                        this.on_click(|event, cx| {
                            if event.up.click_count == 2 {
                                cx.zoom_window();
                            }
                        })
                    })
                    // left side
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .on_mouse_move(|_, cx| cx.stop_propagation())
                            .child("GPUI App"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_end()
                            .px_2()
                            .on_mouse_move(|_, cx| cx.stop_propagation())
                            .child(
                                Switch::new("theme-mode")
                                    .size(ButtonSize::Small)
                                    .checked(cx.theme().mode.is_dark())
                                    .label_side(LabelSide::Left)
                                    .label("Dark Mode")
                                    .on_click(move |_, cx| {
                                        let mode = match cx.theme().mode.is_dark() {
                                            false => ui::theme::ThemeMode::Dark,
                                            true => ui::theme::ThemeMode::Light,
                                        };

                                        Theme::change(mode, cx);
                                    }),
                            ),
                    ),
            )
            .child(self.workspace.clone())
    }
}
