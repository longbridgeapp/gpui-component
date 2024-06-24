use std::{ops::Range, time::Duration};

use gpui::{
    div, px, relative, ClipboardItem, Context, Element, EventEmitter, FocusHandle, HighlightStyle,
    InteractiveElement, InteractiveText, IntoElement, KeyDownEvent, Model, ParentElement, Pixels,
    Render, Style, Styled, StyledText, TextStyle, View, ViewContext, VisualContext, WindowContext,
};

use crate::{disableable::Disableable, hls, theme::Theme};

use super::{
    blink_manager::BlinkManager, cursor_layout::CursorLayout, TextEvent, CURSOR_BLINK_INTERVAL,
};

pub struct TextView {
    pub text: String,
    pub placeholder: String,
    pub word_click: (usize, u16),
    pub selection: Range<usize>,
    pub disable: bool,
    pub blink_manager: Model<BlinkManager>,
    pub cursor: CursorLayout,
}

impl EventEmitter<TextEvent> for TextView {}

impl TextView {
    pub fn init(
        cx: &mut WindowContext,
        focus_handle: &FocusHandle,
        placeholder: &str,
        disable: bool,
    ) -> View<Self> {
        let blink_manager = cx.new_model(|cx| BlinkManager::new(CURSOR_BLINK_INTERVAL, cx));

        let cursor = CursorLayout::new(
            gpui::Point::new(gpui::px(0.0), gpui::px(0.0)),
            px(2.0),
            px(20.0),
            hls(212., 92., 45.),
            None,
        );

        let m = Self {
            text: String::new(),
            placeholder: placeholder.to_string(),
            word_click: (0, 0),
            selection: 0..0,
            blink_manager,
            cursor,
            disable,
        };

        let view = cx.new_view(|cx| {
            cx.on_blur(
                focus_handle,
                |view: &mut TextView, cx: &mut ViewContext<'_, TextView>| {
                    view.blink_manager.update(cx, BlinkManager::disable);
                    cx.emit(TextEvent::Blur);
                },
            )
            .detach();

            cx.on_focus(focus_handle, |view, cx| {
                view.blink_manager.update(cx, |bm, cx| {
                    bm.blink_cursor(0, cx);
                });
            })
            .detach();
            m
        });

        cx.subscribe(
            &view,
            move |subscriber, emitter: &TextEvent, cx| match emitter {
                TextEvent::Input { text: _ } => {
                    subscriber.update(cx, |editor, cx| {
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

    fn paint_cursors(&self, layout: &TextLayout, cx: &mut WindowContext) {
        let mut cursor = self.cursor.clone();
        dbg!("--------- paint_cursors", &cursor);
        cursor.paint(layout.content_origin, cx);
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
}

impl Element for TextView {
    type RequestLayoutState = ();

    type PrepaintState = TextLayout;

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn request_layout(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        cx: &mut WindowContext,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = relative(24.).into();

        (cx.request_layout(style, None), ())
    }

    fn prepaint(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        cx: &mut WindowContext,
    ) -> Self::PrepaintState {
        TextLayout {
            content_origin: bounds.origin,
        }
    }

    fn paint(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        layout: &mut Self::PrepaintState,
        cx: &mut WindowContext,
    ) {
        self.paint_cursors(layout, cx);
    }
}

impl Render for TextView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let mut text = self.text.clone();

        let mut style = TextStyle {
            color: theme.text,
            font_family: theme.font_sans.clone(),
            ..Default::default()
        };

        let mut selection_style = HighlightStyle::default();
        let mut color = theme.lavender;
        color.fade_out(0.8);
        selection_style.background_color = Some(color);

        let highlights = vec![(self.char_range_to_text_range(&text), selection_style)];

        let styled_text: StyledText = if text.is_empty() {
            text = self.placeholder.to_string();
            style.color = theme.subtext0;
            StyledText::new(text).with_highlights(&style, highlights)
        } else {
            StyledText::new(text).with_highlights(&style, highlights)
        };

        let view = cx.view().clone();

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
