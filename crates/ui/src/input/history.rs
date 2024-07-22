use std::{
    fmt::Debug,
    ops::Range,
    time::{Duration, Instant},
};

const MAX_UNDO: usize = 1000;
/// Group interval in milliseconds
const GROUP_INTERVAL: u64 = 500;

#[derive(Debug)]
pub struct History {
    undos: Vec<Change>,
    redos: Vec<Change>,
    last_changed_at: Instant,
    version: usize,
    pub(crate) ignore: bool,
}

#[derive(Debug, Clone)]
pub struct Change {
    pub(crate) old_range: Range<usize>,
    pub(crate) old_text: String,
    pub(crate) new_range: Range<usize>,
    pub(crate) new_text: String,
    version: usize,
}

impl History {
    pub fn new() -> Self {
        Self {
            undos: Default::default(),
            redos: Default::default(),
            ignore: false,
            last_changed_at: Instant::now(),
            version: 0,
        }
    }

    /// Increment the version number if the last change was made more than `GROUP_INTERVAL` milliseconds ago.
    fn inc_version(&mut self) -> usize {
        let t = Instant::now();
        if self.last_changed_at.elapsed().as_millis()
            > Duration::from_millis(GROUP_INTERVAL).as_millis()
        {
            self.version += 1;
        }

        self.last_changed_at = t;
        self.version
    }

    pub fn push(
        &mut self,
        old_range: Range<usize>,
        old_text: &str,
        new_range: Range<usize>,
        new_text: &str,
    ) {
        let version = self.inc_version();

        if self.undos.len() >= MAX_UNDO {
            self.undos.remove(0);
        }
        self.undos.push(Change {
            old_range,
            old_text: old_text.to_string(),
            new_range,
            new_text: new_text.to_string(),
            version,
        });
    }

    pub fn undo(&mut self) -> Option<Vec<Change>> {
        if let Some(first_change) = self.undos.pop() {
            let mut changes = vec![first_change.clone()];
            // pick the next all changes with the same version
            while self
                .undos
                .iter()
                .filter(|c| c.version == first_change.version)
                .count()
                > 0
            {
                let change = self.undos.pop().unwrap();
                changes.push(change);
            }

            self.redos.extend(changes.iter().rev().cloned());
            Some(changes)
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<Vec<Change>> {
        if let Some(first_change) = self.redos.pop() {
            let mut changes = vec![first_change.clone()];
            // pick the next all changes with the same version
            while self
                .redos
                .iter()
                .filter(|c| c.version == first_change.version)
                .count()
                > 0
            {
                let change = self.redos.pop().unwrap();
                changes.push(change);
            }
            self.undos.extend(changes.iter().rev().cloned());
            Some(changes)
        } else {
            None
        }
    }
}
