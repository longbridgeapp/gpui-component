use anyhow::{Context, Result};
use gpui::*;
use std::{sync::Arc, time::Duration};
use story::{ButtonStory, InputStory, StoryContainer};
use ui::{
    theme::ActiveTheme,
    tiles::{CanvasArea, CanvasAreaState, CanvasEvent, CanvasItem},
    Root, TitleBar,
};

use crate::app_state::AppState;

const TILES_CANVAS_AREA: CanvasAreaTab = CanvasAreaTab {
    id: "story-tiles",
    version: 1,
};

actions!(workspace, [Open, CloseWindow]);

pub fn init(_app_state: Arc<AppState>, cx: &mut AppContext) {
    cx.on_action(|_action: &Open, _cx: &mut AppContext| {});

    ui::init(cx);
    story::init(cx);
}

pub struct StoryTiles {
    canvas_area: View<CanvasArea>,
    last_layout_state: Option<CanvasAreaState>,
    _save_layout_task: Option<Task<()>>,
}

struct CanvasAreaTab {
    id: &'static str,
    version: usize,
}

impl StoryTiles {
    pub fn new(_app_state: Arc<AppState>, cx: &mut ViewContext<Self>) -> Self {
        let canvas_area = cx.new_view(|cx| {
            CanvasArea::new(TILES_CANVAS_AREA.id, Some(TILES_CANVAS_AREA.version), cx)
        });
        let weak_canvas_area = canvas_area.downgrade();

        match Self::load_tiles(canvas_area.clone(), cx) {
            Ok(_) => {
                println!("load tiles success");
            }
            Err(err) => {
                eprintln!("load tiles error: {:?}", err);
                Self::reset_default_layout(weak_canvas_area, cx);
            }
        };

        cx.subscribe(
            &canvas_area,
            |this, canvas_area, ev: &CanvasEvent, cx| match ev {
                CanvasEvent::LayoutChanged => this.save_layout(canvas_area, cx),
            },
        )
        .detach();

        cx.on_app_quit({
            let canvas_area = canvas_area.clone();
            move |cx| {
                let state = canvas_area.read(cx).dump(cx);
                cx.background_executor().spawn(async move {
                    // Save layout before quitting
                    Self::save_tiles(&state).unwrap();
                })
            }
        })
        .detach();

        Self {
            canvas_area,
            last_layout_state: None,
            _save_layout_task: None,
        }
    }

    fn save_layout(&mut self, canvas_area: View<CanvasArea>, cx: &mut ViewContext<Self>) {
        self._save_layout_task = Some(cx.spawn(|this, mut cx| async move {
            Timer::after(Duration::from_secs(10)).await;

            let _ = cx.update(|cx| {
                let canvas_area = canvas_area.read(cx);
                let state = canvas_area.dump(cx);

                let last_layout_state = this.upgrade().unwrap().read(cx).last_layout_state.clone();
                if Some(&state) == last_layout_state.as_ref() {
                    return;
                }

                Self::save_tiles(&state).unwrap();
                let _ = this.update(cx, |this, _| {
                    this.last_layout_state = Some(state);
                });
            });
        }));
    }

    fn save_tiles(state: &CanvasAreaState) -> Result<()> {
        println!("Save tiles...");
        let json = serde_json::to_string_pretty(state)?;
        std::fs::write("target/tiles.json", json)?;
        Ok(())
    }

    fn load_tiles(canvas_area: View<CanvasArea>, cx: &mut WindowContext) -> Result<()> {
        let fname = "target/tiles.json";
        let json = std::fs::read_to_string(fname)?;
        let state = serde_json::from_str::<CanvasAreaState>(&json)?;

        // Check if the saved layout version is different from the current version
        // Notify the user and ask if they want to reset the layout to default.
        if state.version != Some(TILES_CANVAS_AREA.version) {
            let answer = cx.prompt(PromptLevel::Info, "The default tiles layout has been updated.\nDo you want to reset the layout to default?", None,
                &["Yes", "No"]);

            let weak_canvas_area = canvas_area.downgrade();
            cx.spawn(|mut cx| async move {
                if answer.await == Ok(0) {
                    _ = cx.update(|cx| {
                        Self::reset_default_layout(weak_canvas_area, cx);
                    });
                }
            })
            .detach();
        }

        canvas_area.update(cx, |canvas_area, cx| {
            canvas_area.load(state, cx).context("load layout")?;

            Ok::<(), anyhow::Error>(())
        })
    }

    fn reset_default_layout(canvas_area: WeakView<CanvasArea>, cx: &mut WindowContext) {
        let canvas_item = Self::init_default_layout(&canvas_area, cx);
        _ = canvas_area.update(cx, |view, cx| {
            view.set_version(TILES_CANVAS_AREA.version, cx);
            view.set_center(canvas_item, cx);

            Self::save_tiles(&view.dump(cx)).unwrap();
        });
    }

    fn init_default_layout(
        canvas_area: &WeakView<CanvasArea>,
        cx: &mut WindowContext,
    ) -> CanvasItem {
        CanvasItem::tiles_with_sizes(
            vec![
                (
                    CanvasItem::tabs(
                        vec![Arc::new(StoryContainer::panel::<ButtonStory>(cx))],
                        None,
                        canvas_area,
                        cx,
                    ),
                    Bounds::new(point(px(20.), px(20.)), size(px(610.), px(190.))),
                    1,
                ),
                (
                    CanvasItem::tabs(
                        vec![Arc::new(StoryContainer::panel::<InputStory>(cx))],
                        None,
                        canvas_area,
                        cx,
                    ),
                    Bounds::new(point(px(120.), px(230.)), size(px(650.), px(300.))),
                    2,
                ),
            ],
            canvas_area,
            cx,
        )
    }

    pub fn new_local(
        app_state: Arc<AppState>,
        cx: &mut AppContext,
    ) -> Task<anyhow::Result<WindowHandle<Root>>> {
        let window_bounds = Bounds::centered(None, size(px(800.0), px(600.0)), cx);

        cx.spawn(|mut cx| async move {
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(window_bounds)),
                titlebar: Some(TitlebarOptions {
                    title: None,
                    appears_transparent: true,
                    traffic_light_position: Some(point(px(9.0), px(9.0))),
                }),
                window_min_size: Some(gpui::Size {
                    width: px(640.),
                    height: px(480.),
                }),
                kind: WindowKind::Normal,
                ..Default::default()
            };

            let window = cx.open_window(options, |cx| {
                let tiles_view = cx.new_view(|cx| Self::new(app_state.clone(), cx));
                cx.new_view(|cx| Root::new(tiles_view.into(), cx))
            })?;

            window
                .update(&mut cx, |_, cx| {
                    cx.activate_window();
                    cx.set_window_title("Story Tiles");
                })
                .expect("failed to update window");

            Ok(window)
        })
    }
}

pub fn open_new(
    app_state: Arc<AppState>,
    cx: &mut AppContext,
    init: impl FnOnce(&mut Root, &mut ViewContext<Root>) + 'static + Send,
) -> Task<()> {
    let task: Task<std::result::Result<WindowHandle<Root>, anyhow::Error>> =
        StoryTiles::new_local(app_state, cx);
    cx.spawn(|mut cx| async move {
        if let Some(root) = task.await.ok() {
            root.update(&mut cx, |workspace, cx| init(workspace, cx))
                .expect("failed to init workspace");
        }
    })
}

impl Render for StoryTiles {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let drawer_layer = Root::render_drawer_layer(cx);
        let modal_layer = Root::render_modal_layer(cx);
        let notification_layer = Root::render_notification_layer(cx);

        div()
            .font_family(".SystemUIFont")
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(TitleBar::new().child(div().flex().items_center().child("Story Tiles")))
            .child(self.canvas_area.clone())
            .children(drawer_layer)
            .children(modal_layer)
            .child(div().absolute().top_8().children(notification_layer))
    }
}
