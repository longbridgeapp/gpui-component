use std::ops::Range;

use gpui::{ModelContext, WeakView};

pub struct History {
    undos: Vec<Change>,
    redos: Vec<Change>,
}

#[derive(Debug, Clone)]
pub enum Change {
    Edit(EditChange),
    Undo,
}

#[derive(Debug, Clone)]
pub struct EditChange {
    pub(crate) range_utf16: Range<usize>,
    pub(crate) text: String,
}

impl History {
    pub fn new() -> Self {
        Self {
            undos: Default::default(),
            redos: Default::default(),
        }
    }

    pub fn push(&mut self, change: Change) {
        self.undos.push(change)
    }

    pub fn undo(&mut self) -> Option<Change> {
        if let Some(change) = self.undos.pop() {
            self.redos.push(change.clone());
            Some(change)
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<Change> {
        if let Some(change) = self.redos.pop() {
            self.undos.push(change.clone());
            Some(change)
        } else {
            None
        }
    }
}
