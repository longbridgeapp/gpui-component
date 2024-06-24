use std::ops::Range;

use super::{
    blink_manager::BlinkManager, cursor_layout::CursorLayout, TextEvent, CURSOR_BLINK_INTERVAL,
};
use crate::theme::{Colorize as _, Theme};
use catppuccin::Hsl;
use gpui::{
    px, relative, ContentMask, Context, Element, EventEmitter, FocusHandle, HighlightStyle, Hsla,
    InteractiveText, IntoElement, Model, Point, Render, Style, StyledText, TextStyle,
    TextStyleRefinement, View, ViewContext, VisualContext, WindowContext,
};

#[derive(Clone)]
pub struct TextFieldStyle {
    pub background: Hsla,
    pub text: TextStyle,
}

pub struct TextView {
    pub text: String,
    pub style: TextFieldStyle,
    pub placeholder: String,
    pub word_click: (usize, u16),
    pub selection: Range<usize>,
    pub disabled: bool,
    pub blink_manager: Model<BlinkManager>,
    pub masked: bool,
    pub focused: bool,
}

impl EventEmitter<TextEvent> for TextView {}

impl TextView {
    pub fn init(cx: &mut WindowContext, focus_handle: &FocusHandle) -> View<Self> {
        let blink_manager = cx.new_model(|cx| BlinkManager::new(CURSOR_BLINK_INTERVAL, cx));

        let theme = cx.global::<Theme>();

        let line_height = px(20.0);
        let style = TextFieldStyle {
            background: theme.transparent,
            text: TextStyle {
                color: theme.foreground,
                line_height: line_height.into(),
                ..Default::default()
            },
        };

        let m = Self {
            text: String::new(),
            style,
            placeholder: "".to_string(),
            word_click: (0, 0),
            selection: 0..0,
            blink_manager: blink_manager.clone(),
            disabled: false,
            masked: false,
            focused: false,
        };

        let view = cx.new_view(|cx| {
            cx.on_blur(focus_handle, |view: &mut TextView, cx| {
                view.blur(cx);
            })
            .detach();

            cx.on_focus(focus_handle, |view, cx| {
                view.focus(cx);
            })
            .detach();

            cx.observe(&blink_manager, |_, _, cx| cx.notify()).detach();

            cx.observe_window_activation(|view, cx| {
                let active = cx.is_window_active();
                view.blink_manager.update(cx, |blink_manager, cx| {
                    if active {
                        blink_manager.enable(cx);
                    } else {
                        blink_manager.show_cursor(cx);
                        blink_manager.disable(cx);
                    }
                })
            })
            .detach();

            m
        });

        cx.subscribe(
            &view,
            move |subscriber, emitter: &TextEvent, cx| match emitter {
                TextEvent::Input { text: _ } => {
                    subscriber.update(cx, |editor, _cx| {
                        editor.word_click = (0, 0);
                    });
                }
                TextEvent::Blur => {
                    subscriber.update(cx, |editor, cx| {
                        editor.blink_manager.update(cx, BlinkManager::disable);
                        editor.word_click = (0, 0);
                    });
                }
                _ => {}
            },
        )
        .detach();

        view
    }

    pub fn select_all(&mut self, cx: &mut ViewContext<Self>) {
        self.selection = 0..self.text.len();
        cx.notify();
    }

    pub fn blur(&mut self, cx: &mut ViewContext<Self>) {
        self.focused = false;
        self.blink_manager.update(cx, BlinkManager::disable);
        cx.notify();
        cx.emit(TextEvent::Blur);
    }

    pub fn focus(&mut self, cx: &mut ViewContext<Self>) {
        self.focused = true;
        self.blink_manager.update(cx, |bm, cx| {
            bm.blink_cursor(0, cx);
        });
        cx.notify();
        cx.emit(TextEvent::Focus);
    }

    pub fn word_ranges(&self) -> Vec<Range<usize>> {
        let mut words = Vec::new();
        let mut last_was_boundary = true;
        let mut word_start = 0;
        let s = self.text.clone();

        for (i, c) in s.char_indices() {
            if c.is_alphanumeric() || c == '_' {
                if last_was_boundary {
                    word_start = i;
                }
                last_was_boundary = false;
            } else {
                if !last_was_boundary {
                    words.push(word_start..i);
                }
                last_was_boundary = true;
            }
        }

        // Check if the last characters form a word and push it if so
        if !last_was_boundary {
            words.push(word_start..s.len());
        }

        words
    }

    /// Converts a character range to a text range (in bytes)
    pub fn char_range_to_text_range(&self, text: &str) -> Range<usize> {
        let start = text
            .chars()
            .take(self.selection.start)
            .collect::<String>()
            .len();
        let end = text
            .chars()
            .take(self.selection.end)
            .collect::<String>()
            .len();
        start..end
    }

    pub fn set_text(&mut self, text: impl ToString, cx: &mut ViewContext<Self>) {
        self.text = text.to_string();
        self.selection = self.text.len()..self.text.len();
        cx.notify();
        cx.emit(TextEvent::Input {
            text: self.text.clone(),
        });
    }

    pub fn set_masked(&mut self, masked: bool, cx: &mut ViewContext<Self>) {
        self.masked = masked;
        cx.notify();
    }

    pub fn set_placeholder(&mut self, placeholder: impl ToString, cx: &mut ViewContext<Self>) {
        self.placeholder = placeholder.to_string();
        cx.notify();
    }

    pub fn set_disabled(&mut self, disabled: bool, cx: &mut ViewContext<Self>) {
        self.disabled = disabled;
        cx.notify();
    }

    fn paint_cursors(&self, layout: &TextLayout, cx: &mut WindowContext) {
        let mut cursor = layout.visible_cursor.clone();
        cursor.paint(layout.content_origin, cx);
    }

    pub fn show_cursor(&self, cx: &mut WindowContext) -> bool {
        self.blink_manager.read(cx).visible() && self.focused
    }

    fn layout_visible_cursors(&self, cx: &mut WindowContext) -> CursorLayout {
        let theme = cx.global::<Theme>();
        let selection = &self.selection;

        let x = px(selection.end as f32);
        let y = px(0.);

        CursorLayout::new(Point::new(x, y), px(0.), px(20.0), theme.ring, None)
    }
}

impl IntoElement for TextView {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

pub struct TextLayout {
    content_origin: gpui::Point<gpui::Pixels>,
    visible_cursor: CursorLayout,
}

impl Element for TextView {
    type RequestLayoutState = ();

    type PrepaintState = TextLayout;

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        cx: &mut WindowContext,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        let rem_size = cx.rem_size();

        style.size.width = relative(1.).into();
        style.size.height = self.style.text.line_height_in_pixels(rem_size).into();

        (cx.request_layout(style, None), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        cx: &mut WindowContext,
    ) -> Self::PrepaintState {
        let text_style = TextStyleRefinement {
            font_size: Some(self.style.text.font_size),
            line_height: Some(self.style.text.line_height),
            ..Default::default()
        };

        cx.with_text_style(Some(text_style), |cx| {
            cx.with_content_mask(Some(ContentMask { bounds }), |cx| {
                let cursor = self.layout_visible_cursors(cx);

                TextLayout {
                    content_origin: bounds.origin,
                    visible_cursor: cursor,
                }
            })
        })
    }

    fn paint(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        layout: &mut Self::PrepaintState,
        cx: &mut WindowContext,
    ) {
        let text_style = TextStyleRefinement {
            font_size: Some(self.style.text.font_size),
            line_height: Some(self.style.text.line_height),
            ..Default::default()
        };

        cx.with_text_style(Some(text_style), |cx| {
            cx.with_content_mask(Some(ContentMask { bounds }), |cx| {
                self.paint_cursors(layout, cx);
            })
        });
    }
}

impl Render for TextView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();

        let view = cx.view().clone();
        let mut text = self.text.clone();

        let mut style = self.style.text.clone();
        if self.masked {
            text = "•".repeat(text.len());
        }

        let mut selection_style = HighlightStyle::default();
        selection_style.background_color = Some(theme.ring);
        selection_style.color = Some(theme.ring.invert());

        let mut highlights = vec![(self.char_range_to_text_range(&text), selection_style)];

        if text.is_empty() {
            text = self.placeholder.to_string();
            style.color = theme.muted_foreground;
        }

        if !self.focused {
            highlights = vec![];
        }

        let styled_text = StyledText::new(text).with_highlights(&style, highlights);

        InteractiveText::new("text", styled_text).on_click(self.word_ranges(), move |ev, cx| {
            view.update(cx, |text_view, cx| {
                let (index, mut count) = text_view.word_click;
                if index == ev {
                    count += 1;
                } else {
                    count = 1;
                }
                match count {
                    2 => {
                        let word_ranges = text_view.word_ranges();
                        text_view.selection = word_ranges.get(ev).unwrap().clone();
                    }
                    3 => {
                        // Should select the line
                    }
                    4 => {
                        count = 0;
                        text_view.selection = 0..text_view.text.len();
                    }
                    _ => {}
                }
                text_view.word_click = (ev, count);
                cx.notify();
            });
        })
    }
}
