use std::time::Duration;

use gpui::ModelContext;

pub struct BlinkManager {
    blink_interval: Duration,
    blink_epoch: usize,
    blinking_paused: bool,
    visible: bool,
    enabled: bool,
}

impl BlinkManager {
    pub fn new(blink_interval: Duration, _cx: &mut ModelContext<Self>) -> Self {
        Self {
            blink_interval,
            blink_epoch: 0,
            blinking_paused: false,
            visible: true,
            enabled: true,
        }
    }

    pub fn show_cursor(&self, _cx: &mut ModelContext<'_, Self>) -> bool {
        self.enabled && (!self.blinking_paused || self.visible)
    }

    pub fn blink_cursor(&mut self, epoch: usize, cx: &mut ModelContext<Self>) {
        if self.blink_epoch != epoch {
            self.blink_epoch = epoch;
            self.visible = !self.visible;
            cx.refresh();
        }
    }

    pub fn disable(&mut self, _cx: &mut ModelContext<Self>) {
        self.enabled = false;
    }

    pub fn enable(&mut self, _cx: &mut ModelContext<Self>) {
        self.enabled = true;
    }

    pub fn visible(&self) -> bool {
        self.visible
    }
}
