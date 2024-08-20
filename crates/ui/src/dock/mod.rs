mod dock;
mod panel;
mod stack_panel;
mod state;
mod tab_panel;

use gpui::{Styled, WindowContext};
pub(crate) use state::*;

pub use dock::*;
pub use panel::*;
pub use stack_panel::*;
pub use tab_panel::*;

use crate::theme::ActiveTheme;

pub(crate) trait DropTargetStyled: Styled {
    fn drop_target(self, is_droped: bool, cx: &mut WindowContext) -> Self {
        if !is_droped {
            return self;
        }

        self.border_1().bg(cx.theme().drop_target)
    }
}
