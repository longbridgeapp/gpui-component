use gpui::{hsla, prelude::*, Hsla};

use ui::{h_flex, prelude::*, theme::ActiveTheme};

#[derive(IntoElement)]
pub struct WindowsWindowControls {
    button_height: Pixels,
}

impl WindowsWindowControls {
    pub fn new(button_height: Pixels) -> Self {
        Self { button_height }
    }
}

impl RenderOnce for WindowsWindowControls {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let close_button_hover_color = hsla(356.0 / 360.0, 0.86, 0.4, 1.0);

        let button_hover_color = if cx.theme().mode.is_dark() {
            hsla(180.0 / 360.0, 0.01, 0.21, 1.0)
        } else {
            hsla(0.0, 0.0, 0.91, 1.0)
        };

        div()
            .id("windows-window-controls")
            .flex()
            .flex_row()
            .justify_center()
            .content_stretch()
            .max_h(self.button_height)
            .min_h(self.button_height)
            .child(WindowsCaptionButton::new(
                "minimize",
                WindowsCaptionButtonIcon::Minimize,
                button_hover_color,
            ))
            .child(WindowsCaptionButton::new(
                "maximize-or-restore",
                if cx.is_maximized() {
                    WindowsCaptionButtonIcon::Restore
                } else {
                    WindowsCaptionButtonIcon::Maximize
                },
                button_hover_color,
            ))
            .child(WindowsCaptionButton::new(
                "close",
                WindowsCaptionButtonIcon::Close,
                close_button_hover_color,
            ))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
enum WindowsCaptionButtonIcon {
    Minimize,
    Restore,
    Maximize,
    Close,
}

#[derive(IntoElement)]
struct WindowsCaptionButton {
    id: ElementId,
    icon: WindowsCaptionButtonIcon,
    hover_background_color: Hsla,
}

impl WindowsCaptionButton {
    pub fn new(
        id: impl Into<ElementId>,
        icon: WindowsCaptionButtonIcon,
        hover_background_color: Hsla,
    ) -> Self {
        Self {
            id: id.into(),
            icon,
            hover_background_color,
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn get_font() -> &'static str {
        "Segoe Fluent Icons"
    }

    #[cfg(target_os = "windows")]
    fn get_font() -> &'static str {
        use windows::Wdk::System::SystemServices::RtlGetVersion;

        let mut version = unsafe { std::mem::zeroed() };
        let status = unsafe { RtlGetVersion(&mut version) };

        if status.is_ok() && version.dwBuildNumber >= 22000 {
            "Segoe Fluent Icons"
        } else {
            "Segoe MDL2 Assets"
        }
    }
}

impl RenderOnce for WindowsCaptionButton {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        // todo(windows) report this width to the Windows platform API
        // NOTE: this is intentionally hard coded. An option to use the 'native' size
        //       could be added when the width is reported to the Windows platform API
        //       as this could change between future Windows versions.
        let width = px(36.);

        h_flex()
            .id(self.id)
            .justify_center()
            .content_center()
            .w(width)
            .h_full()
            .text_size(px(10.0))
            .font_family(Self::get_font())
            .text_color(cx.theme().foreground)
            .hover(|style| style.bg(self.hover_background_color))
            .active(|style| {
                let mut active_color = self.hover_background_color;
                active_color.l *= 0.95;

                style.bg(active_color)
            })
            .child(match self.icon {
                WindowsCaptionButtonIcon::Minimize => "\u{e921}",
                WindowsCaptionButtonIcon::Restore => "\u{e923}",
                WindowsCaptionButtonIcon::Maximize => "\u{e922}",
                WindowsCaptionButtonIcon::Close => "\u{e8bb}",
            })
    }
}
