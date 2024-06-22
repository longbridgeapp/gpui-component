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
    pub fn new(blink_interval: Duration) -> Self {
        Self {
            blink_interval: Duration::from_millis(500),
            blink_epoch: 0,
            blinking_paused: false,
            visible: true,
            enabled: true,
        }
    }

    pub fn show_cursor(&self, cx: &mut ModelContext<'_, Self>) -> bool {
        self.enabled && (!self.blinking_paused || self.visible)
    }

    pub fn blink_cursor(&mut self, epoch: usize, cx: &mut ModelContext<Self>) {}

    pub fn disable(&mut self, _cx: &mut ModelContext<Self>) {
        self.enabled = false;
    }
}
