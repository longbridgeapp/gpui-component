use std::time::Duration;

use gpui::{ModelContext, Timer};

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

    pub fn pause_blinking(&mut self, cx: &mut ModelContext<Self>) {
        self.show_cursor(cx);

        let epoch = self.next_blink_epoch();
        let interval = self.blink_interval;
        cx.spawn(|this, mut cx| async move {
            Timer::after(interval).await;
            this.update(&mut cx, |this, cx| this.resume_cursor_blinking(epoch, cx))
        })
        .detach();
    }

    fn next_blink_epoch(&mut self) -> usize {
        self.blink_epoch += 1;
        self.blink_epoch
    }

    fn resume_cursor_blinking(&mut self, epoch: usize, cx: &mut ModelContext<Self>) {
        if epoch == self.blink_epoch {
            self.blinking_paused = false;
            self.blink_cursor(epoch, cx);
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
