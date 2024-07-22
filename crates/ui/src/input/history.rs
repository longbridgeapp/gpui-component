use std::{fmt::Debug, ops::Range};

#[derive(Debug)]
pub struct History {
    undos: Vec<Change>,
    redos: Vec<Change>,
    pub(crate) ignore: bool,
}

#[derive(Debug, Clone)]
pub struct Change {
    pub(crate) old_range: Range<usize>,
    pub(crate) old_text: String,
    pub(crate) new_range: Range<usize>,
    pub(crate) new_text: String,
}

impl History {
    pub fn new() -> Self {
        Self {
            undos: Default::default(),
            redos: Default::default(),
            ignore: false,
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

    pub fn can_undo(&self) -> bool {
        !self.undos.is_empty()
    }

    pub fn redo(&mut self) -> Option<Change> {
        if let Some(change) = self.redos.pop() {
            self.undos.push(change.clone());
            Some(change)
        } else {
            None
        }
    }

    pub fn can_redo(&self) -> bool {
        !self.redos.is_empty()
    }
}
