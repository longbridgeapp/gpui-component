use anyhow::Result;
use gpui::*;
use prelude::FluentBuilder as _;
use serde::Deserialize;
use std::{sync::Arc, time::Duration};
use story::{
    ButtonStory, CalendarStory, DropdownStory, IconStory, ImageStory, InputStory, ListStory,
    ModalStory, PopupStory, ProgressStory, ResizableStory, ScrollableStory, StoryContainer,
    SwitchStory, TableStory, TextStory, TooltipStory,
};
use ui::{
    button::{Button, ButtonStyled as _},
    color_picker::{ColorPicker, ColorPickerEvent},
    dock::{DockArea, DockEvent, DockItem, DockItemState},
    h_flex,
    popup_menu::PopupMenuExt,
    theme::{ActiveTheme, Colorize as _, Theme},
    ContextModal, IconName, Root, Sizable,
};
use workspace::TitleBar;

use crate::app_state::AppState;

#[derive(Clone, PartialEq, Eq, Deserialize)]
struct SelectLocale(SharedString);

impl_actions!(locale_switcher, [SelectLocale]);

actions!(workspace, [Open, CloseWindow]);

pub fn init(_app_state: Arc<AppState>, cx: &mut AppContext) {
    cx.on_action(|_action: &Open, _cx: &mut AppContext| {});

    ui::init(cx);
    story::init(cx);
}

pub struct StoryWorkspace {
    dock_area: View<DockArea>,
    locale_selector: View<LocaleSelector>,
    theme_color_picker: View<ColorPicker>,
    last_layout_state: Option<DockItemState>,
    _save_layout_task: Option<Task<()>>,
}

impl StoryWorkspace {
    pub fn new(_app_state: Arc<AppState>, cx: &mut ViewContext<Self>) -> Self {
        cx.observe_window_appearance(|_workspace, cx| {
            Theme::sync_system_appearance(cx);
        })
        .detach();

        let dock_area = cx.new_view(|cx| DockArea::new("main-dock", cx));
        let weak_dock_area = dock_area.downgrade();

        let dock_item = match Self::load_layout(&weak_dock_area, cx) {
            Ok(item) => item,
            Err(err) => {
                eprintln!("load layout error: {:?}", err);
                Self::init_default_layout(&weak_dock_area, cx)
            }
        };

        dock_area.update(cx, |view, cx| view.set_root(dock_item, cx));

        cx.subscribe(&dock_area, |this, dock_area, ev: &DockEvent, cx| match ev {
            DockEvent::LayoutChanged => this.save_layout(dock_area, cx),
        })
        .detach();

        let dock_area1 = dock_area.clone();
        cx.on_app_quit(move |cx| {
            let state = dock_area1.read(cx).dump(cx);

            cx.background_executor().spawn(async move {
                // Save layout before quitting
                Self::save_state(&state).unwrap();
            })
        })
        .detach();

        let locale_selector = cx.new_view(LocaleSelector::new);

        let theme_color_picker = cx.new_view(|cx| {
            let mut picker = ColorPicker::new("theme-color-picker", cx)
                .xsmall()
                .anchor(AnchorCorner::TopRight)
                .label("Primary Color");
            picker.set_value(cx.theme().primary, cx);
            picker
        });
        cx.subscribe(
            &theme_color_picker,
            |_, _, ev: &ColorPickerEvent, cx| match ev {
                ColorPickerEvent::Change(color) => {
                    if let Some(color) = color {
                        let theme = cx.global_mut::<Theme>();
                        theme.primary = *color;
                        theme.primary_hover = color.lighten(0.1);
                        theme.primary_active = color.darken(0.1);
                        cx.refresh();
                    }
                }
            },
        )
        .detach();

        Self {
            dock_area,
            locale_selector,
            theme_color_picker,
            last_layout_state: None,
            _save_layout_task: None,
        }
    }

    fn save_layout(&mut self, dock_area: View<DockArea>, cx: &mut ViewContext<Self>) {
        self._save_layout_task = Some(cx.spawn(|this, mut cx| async move {
            Timer::after(Duration::from_secs(1)).await;

            let _ = cx.update(|cx| {
                let dock_area = dock_area.read(cx);
                let state = dock_area.dump(cx);

                let last_layout_state = this.upgrade().unwrap().read(cx).last_layout_state.clone();
                if Some(&state) == last_layout_state.as_ref() {
                    return;
                }

                Self::save_state(&state).unwrap();
                let _ = this.update(cx, |this, _| {
                    this.last_layout_state = Some(state);
                });
            });
        }));
    }

    fn save_state(state: &DockItemState) -> Result<()> {
        println!("Save layout...");
        let json = serde_json::to_string_pretty(state)?;
        std::fs::write("layout.json", json)?;
        Ok(())
    }

    fn load_layout(dock_area: &WeakView<DockArea>, cx: &mut WindowContext) -> Result<DockItem> {
        let fname = "layout.json";
        let json = std::fs::read_to_string(fname)?;
        let state = serde_json::from_str::<DockItemState>(&json)?;

        return Ok(state.to_item(dock_area.clone(), cx));
    }

    fn init_default_layout(dock_area: &WeakView<DockArea>, cx: &mut WindowContext) -> DockItem {
        DockItem::split_with_sizes(
            Axis::Horizontal,
            vec![
                DockItem::split(
                    Axis::Vertical,
                    vec![
                        DockItem::tab(StoryContainer::panel::<IconStory>(cx), &dock_area, cx),
                        DockItem::tab(StoryContainer::panel::<CalendarStory>(cx), &dock_area, cx),
                    ],
                    &dock_area,
                    cx,
                ),
                DockItem::split_with_sizes(
                    Axis::Vertical,
                    vec![
                        DockItem::tabs(
                            vec![
                                Arc::new(StoryContainer::panel::<ButtonStory>(cx)),
                                Arc::new(StoryContainer::panel::<InputStory>(cx)),
                                Arc::new(StoryContainer::panel::<DropdownStory>(cx)),
                                Arc::new(StoryContainer::panel::<ModalStory>(cx)),
                                Arc::new(StoryContainer::panel::<PopupStory>(cx)),
                                Arc::new(StoryContainer::panel::<ListStory>(cx)),
                                Arc::new(StoryContainer::panel::<SwitchStory>(cx)),
                                Arc::new(StoryContainer::panel::<ProgressStory>(cx)),
                                Arc::new(StoryContainer::panel::<TableStory>(cx)),
                                Arc::new(StoryContainer::panel::<ImageStory>(cx)),
                                Arc::new(StoryContainer::panel::<ResizableStory>(cx)),
                                Arc::new(StoryContainer::panel::<ScrollableStory>(cx)),
                            ],
                            None,
                            &dock_area,
                            cx,
                        ),
                        DockItem::tabs(
                            vec![
                                Arc::new(StoryContainer::panel::<ProgressStory>(cx)),
                                Arc::new(StoryContainer::panel::<TextStory>(cx)),
                            ],
                            None,
                            &dock_area,
                            cx,
                        ),
                    ],
                    vec![None, None, Some(px(300.))],
                    &dock_area,
                    cx,
                ),
                DockItem::split_with_sizes(
                    Axis::Vertical,
                    vec![
                        DockItem::tab(StoryContainer::panel::<TooltipStory>(cx), &dock_area, cx),
                        DockItem::tab(StoryContainer::panel::<CalendarStory>(cx), &dock_area, cx),
                        DockItem::tab(StoryContainer::panel::<ImageStory>(cx), &dock_area, cx),
                    ],
                    vec![None, None, Some(px(300.))],
                    &dock_area,
                    cx,
                ),
            ],
            vec![Some(px(300.)), None, Some(px(350.))],
            &dock_area,
            cx,
        )
    }

    pub fn new_local(
        app_state: Arc<AppState>,
        cx: &mut AppContext,
    ) -> Task<anyhow::Result<WindowHandle<Root>>> {
        let window_bounds = Bounds::centered(None, size(px(1600.0), px(1200.0)), cx);

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
                let story_view = cx.new_view(|cx| Self::new(app_state.clone(), cx));
                cx.new_view(|cx| Root::new(story_view.into(), cx))
            })?;

            window
                .update(&mut cx, |_, cx| {
                    cx.activate_window();
                    cx.set_window_title("GPUI App");
                    cx.on_release(|_, _, cx| {
                        // exit app
                        cx.quit();
                    })
                    .detach();
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
        StoryWorkspace::new_local(app_state, cx);
    cx.spawn(|mut cx| async move {
        if let Some(root) = task.await.ok() {
            root.update(&mut cx, |workspace, cx| init(workspace, cx))
                .expect("failed to init workspace");
        }
    })
}

impl Render for StoryWorkspace {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let drawer_layer = Root::render_drawer_layer(cx);
        let modal_layer = Root::render_modal_layer(cx);
        let notification_layer = Root::render_notification_layer(cx);
        let notifications_count = cx.notifications().len();

        div()
            .font_family(".SystemUIFont")
            .relative()
            .size_full()
            .flex()
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
                    .child(div().flex().items_center().child("GPUI App"))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_end()
                            .px_2()
                            .gap_2()
                            .child(self.theme_color_picker.clone())
                            .child(
                                Button::new("theme-mode")
                                    .map(|this| {
                                        if cx.theme().mode.is_dark() {
                                            this.icon(IconName::Sun)
                                        } else {
                                            this.icon(IconName::Moon)
                                        }
                                    })
                                    .small()
                                    .ghost()
                                    .on_click(move |_, cx| {
                                        let mode = match cx.theme().mode.is_dark() {
                                            true => ui::theme::ThemeMode::Light,
                                            false => ui::theme::ThemeMode::Dark,
                                        };

                                        Theme::change(mode, cx);
                                    }),
                            )
                            .child(self.locale_selector.clone())
                            .child(
                                Button::new("github")
                                    .icon(IconName::GitHub)
                                    .small()
                                    .ghost()
                                    .on_click(|_, cx| {
                                        cx.open_url("https://github.com/huacnlee/gpui-component")
                                    }),
                            )
                            .child(
                                div()
                                    .relative()
                                    .child(
                                        Button::new("bell")
                                            .small()
                                            .ghost()
                                            .compact()
                                            .icon(IconName::Bell),
                                    )
                                    .when(notifications_count > 0, |this| {
                                        this.child(
                                            h_flex()
                                                .absolute()
                                                .rounded_full()
                                                .top(px(-2.))
                                                .right(px(-2.))
                                                .p(px(1.))
                                                .min_w(px(12.))
                                                .bg(ui::red_500())
                                                .text_color(ui::white())
                                                .justify_center()
                                                .text_size(px(10.))
                                                .line_height(relative(1.))
                                                .child(format!("{}", notifications_count.min(99))),
                                        )
                                    }),
                            ),
                    ),
            )
            .child(self.dock_area.clone())
            .children(drawer_layer)
            .children(modal_layer)
            .child(div().absolute().top_8().children(notification_layer))
    }
}

struct LocaleSelector {
    focus_handle: FocusHandle,
}

impl LocaleSelector {
    pub fn new(cx: &mut ViewContext<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }

    fn on_select_locale(&mut self, locale: &SelectLocale, cx: &mut ViewContext<Self>) {
        ui::set_locale(&locale.0);
        cx.refresh();
    }
}

impl Render for LocaleSelector {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle.clone();
        let locale = ui::locale().to_string();

        div()
            .id("locale-selector")
            .track_focus(&focus_handle)
            .on_action(cx.listener(Self::on_select_locale))
            .child(
                Button::new("btn")
                    .small()
                    .ghost()
                    .icon(IconName::Globe)
                    .popup_menu(move |this, _| {
                        this.menu_with_check(
                            "English",
                            locale == "en",
                            Box::new(SelectLocale("en".into())),
                        )
                        .menu_with_check(
                            "简体中文",
                            locale == "zh-CN",
                            Box::new(SelectLocale("zh-CN".into())),
                        )
                    })
                    .anchor(AnchorCorner::TopRight),
            )
    }
}
