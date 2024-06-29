use std::time::Duration;

use gpui::{ModelContext, Timer};

/// To manage the Input cursor blinking.
///
/// It will start blinking with a interval of 500ms.
/// Every loop will notify the view to update the `visable`, and Input will observe this update to touch repaint.
///
/// The input painter will check if this in visible state, then it will draw the cursor.
pub struct BlinkCursor {
    interval: Duration,
    blink_epoch: usize,
    visible: bool,
    paused: bool,
    started: bool,
}

impl BlinkCursor {
    pub fn new(_cx: &mut ModelContext<Self>) -> Self {
        Self {
            interval: Duration::from_millis(500),
            visible: false,
            paused: false,
            started: false,
            blink_epoch: 0,
        }
    }

    /// Start the blinking
    pub fn start(&mut self, cx: &mut ModelContext<Self>) {
        if self.started {
            return;
        }

        self.started = true;
        self.paused = false;
        self.blink(self.blink_epoch, cx);
    }

    fn next_epoch(&mut self) -> usize {
        self.blink_epoch += 1;
        self.blink_epoch
    }

    fn blink(&mut self, epoch: usize, cx: &mut ModelContext<Self>) {
        if self.paused {
            return;
        }

        if epoch != self.blink_epoch {
            return;
        }

        self.visible = !self.visible;
        cx.notify();

        let epoch = self.next_epoch();

        // Schedule the next blink
        let interval = self.interval;
        cx.spawn(|this, mut cx| async move {
            Timer::after(interval).await;
            if let Some(this) = this.upgrade() {
                this.update(&mut cx, |this, cx| this.blink(epoch, cx)).ok();
            }
        })
        .detach();
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn pause(&mut self, cx: &mut ModelContext<Self>) {
        self.paused = true;
        self.started = false;
        cx.notify();
    }
}
