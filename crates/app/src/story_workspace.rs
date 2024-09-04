use gpui::*;
use prelude::FluentBuilder as _;
use private::serde::Deserialize;
use story::{
    ButtonStory, CalendarStory, DropdownStory, IconStory, ImageStory, InputStory, ListStory,
    ModalStory, PopupStory, ProgressStory, ResizableStory, ScrollableStory, StoryContainer,
    SwitchStory, TableStory, TextStory, TooltipStory,
};
use workspace::TitleBar;

use std::sync::Arc;
use ui::{
    button::Button,
    color_picker::{ColorPicker, ColorPickerEvent},
    dock::{DockArea, StackPanel, TabPanel},
    drawer::Drawer,
    h_flex,
    modal::Modal,
    popup_menu::PopupMenuExt,
    theme::{ActiveTheme, Colorize as _, Theme},
    ContextModal, IconName, Placement, Root, Sizable,
};

use crate::app_state::AppState;

#[derive(Clone, PartialEq, Eq, Deserialize)]
struct SelectLocale(SharedString);

impl_actions!(locale_switcher, [SelectLocale]);

actions!(workspace, [Open, CloseWindow]);

pub fn init(_app_state: Arc<AppState>, cx: &mut AppContext) {
    cx.on_action(|_action: &Open, _cx: &mut AppContext| {});

    Theme::init(cx);
    ui::init(cx);
    story::init(cx);
}

pub struct StoryWorkspace {
    dock_area: View<DockArea>,
    locale_selector: View<LocaleSelector>,
    theme_color_picker: View<ColorPicker>,
}

impl StoryWorkspace {
    pub fn new(_app_state: Arc<AppState>, cx: &mut ViewContext<Self>) -> Self {
        cx.observe_window_appearance(|_workspace, cx| {
            Theme::sync_system_appearance(cx);
        })
        .detach();

        let stack_panel = cx.new_view(|cx| StackPanel::new(Axis::Horizontal, cx));
        let dock_area = cx.new_view(|cx| DockArea::new("main-dock", stack_panel.clone(), cx));
        let weak_dock_area = dock_area.downgrade();

        let center_tab_panel = cx.new_view(|cx| {
            let stack_panel = cx.new_view(|cx| StackPanel::new(Axis::Vertical, cx));
            TabPanel::new(Some(stack_panel), weak_dock_area.clone(), cx)
        });
        let left_tab_panel = cx.new_view(|cx| {
            let stack_panel = cx.new_view(|cx| StackPanel::new(Axis::Vertical, cx));
            TabPanel::new(Some(stack_panel), weak_dock_area.clone(), cx)
        });

        let right_tab_panel = cx.new_view(|cx| {
            let stack_panel = cx.new_view(|cx| StackPanel::new(Axis::Vertical, cx));
            TabPanel::new(Some(stack_panel), weak_dock_area.clone(), cx)
        });

        stack_panel.update(cx, |view, cx| {
            view.add_panel(
                left_tab_panel.clone(),
                Some(px(300.)),
                weak_dock_area.clone(),
                cx,
            );

            view.add_panel(center_tab_panel.clone(), None, weak_dock_area.clone(), cx);
            view.add_panel(
                right_tab_panel.clone(),
                Some(px(350.)),
                weak_dock_area.clone(),
                cx,
            );
        });

        StoryContainer::add_panel(
            "Buttons",
            "Displays a button or a component that looks like a button.",
            ButtonStory::view(cx).into(),
            center_tab_panel.clone(),
            None,
            None,
            false,
            cx,
        );

        StoryContainer::add_panel(
            "Input",
            "A control that allows the user to input text.",
            InputStory::view(cx).into(),
            center_tab_panel.clone(),
            None,
            None,
            false,
            cx,
        );

        StoryContainer::add_panel(
            "Text",
            "Links, paragraphs, checkboxes, and more.",
            TextStory::view(cx).into(),
            center_tab_panel.clone(),
            Some(Placement::Bottom),
            Some(px(200.)),
            true,
            cx,
        );

        StoryContainer::add_panel(
            "Switch",
            "A control that allows the user to toggle between two states.",
            SwitchStory::view(cx).into(),
            center_tab_panel.clone(),
            None,
            None,
            true,
            cx,
        );

        StoryContainer::add_panel(
            "Dropdowns",
            "Displays a list of options for the user to pick from—triggered by a button.",
            DropdownStory::new(cx).into(),
            center_tab_panel.clone(),
            None,
            None,
            true,
            cx,
        );

        StoryContainer::add_panel(
            "Modal",
            "Modal & Drawer use examples",
            ModalStory::view(cx).into(),
            center_tab_panel.clone(),
            None,
            None,
            true,
            cx,
        );

        StoryContainer::add_panel(
            "Popup",
            "A popup displays content on top of the main page.",
            PopupStory::view(cx).into(),
            center_tab_panel.clone(),
            None,
            None,
            true,
            cx,
        );

        StoryContainer::add_panel(
            "Tooltip",
            "Displays a short message when users hover over an element.",
            TooltipStory::view(cx).into(),
            right_tab_panel.clone(),
            None,
            None,
            true,
            cx,
        );

        StoryContainer::add_panel(
            "List",
            "A list displays a series of items.",
            ListStory::view(cx).into(),
            left_tab_panel.clone(),
            None,
            None,
            true,
            cx,
        );

        StoryContainer::add_panel(
            "Icon",
            "Icon use examples",
            IconStory::view(cx).into(),
            left_tab_panel.clone(),
            Some(Placement::Bottom),
            Some(px(200.)),
            true,
            cx,
        );

        StoryContainer::add_panel(
            "Image",
            "Render SVG image and Chart",
            ImageStory::view(cx).into(),
            right_tab_panel.clone(),
            Some(Placement::Bottom),
            None,
            true,
            cx,
        );

        // StoryContainer::add_panel(
        //     WebViewStory::view(cx).into(),
        //     stack_panel.clone(),
        //     DockPosition::Right,
        //     px(450.),
        //     cx,
        // );

        StoryContainer::add_panel(
            "Table",
            "Powerful table and datagrids built.",
            TableStory::view(cx).into(),
            center_tab_panel.clone(),
            None,
            None,
            true,
            cx,
        );

        StoryContainer::add_panel(
            "Progress",
            "Displays an indicator showing the completion progress of a task, typically displayed as a progress bar.",
            ProgressStory::view(cx).into(),
            center_tab_panel.clone(),
            Some(Placement::Bottom),
            Some(px(200.)),
            true,
            cx,
        );

        StoryContainer::add_panel(
            "Resizable",
            "Accessible resizable panel groups and layouts with keyboard support.",
            ResizableStory::view(cx).into(),
            center_tab_panel.clone(),
            None,
            None,
            true,
            cx,
        );

        StoryContainer::add_panel(
            "Scrollable",
            "A scrollable area with scroll bar.",
            ScrollableStory::view(cx).into(),
            center_tab_panel.clone(),
            None,
            None,
            true,
            cx,
        );

        StoryContainer::add_panel(
            "Calendar",
            "A calendar component.",
            CalendarStory::view(cx).into(),
            right_tab_panel.clone(),
            Some(Placement::Bottom),
            None,
            true,
            cx,
        );

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
        }
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
        let active_modal = Root::read(cx).active_modal.clone();
        let active_drawer = Root::read(cx).active_drawer.clone();
        let has_active_modal = active_modal.is_some();
        let notification_view = Root::read(cx).notification.clone();
        let notifications_count = cx.notifications().len();

        div()
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
                                Button::new("theme-mode", cx)
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
                                Button::new("github", cx)
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
                                        Button::new("bell", cx)
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
            .when(!has_active_modal, |this| {
                this.when_some(active_drawer, |this, builder| {
                    let drawer = Drawer::new(cx);
                    this.child(builder(drawer, cx))
                })
            })
            .when_some(active_modal, |this, builder| {
                let modal = Modal::new(cx);
                this.child(builder(modal, cx))
            })
            .child(div().absolute().top_8().child(notification_view))
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
                Button::new("btn", cx)
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
