use gpui::{Pixels, Size};
use serde::{Deserialize, Serialize};

use super::Split;

#[derive(Clone, Serialize, Deserialize)]
pub enum Node<Tab> {
    Empty,
    Leaf {
        /// TabBar + Content bounds
        size: Size<Pixels>,
        /// Content view bounds
        content_size: Size<Pixels>,
        tabs: Vec<Tab>,
        active: usize,
    },
    Vertical {
        /// TabBar + Content bounds
        size: Size<Pixels>,
        /// The fraction taken by the top child of this node.
        fraction: f32,
    },
    Horizontal {
        /// TabBar + Content bounds
        size: Size<Pixels>,
        /// The fraction taken by the left child of this node.
        fraction: f32,
    },
}

impl<Tab> Node<Tab> {
    pub fn leaf(tab: Tab) -> Self {
        Self::Leaf {
            size: Size::default(),
            content_size: Size::default(),
            tabs: vec![tab],
            active: 0,
        }
    }

    pub fn leaf_with(tabs: Vec<Tab>) -> Self {
        Self::Leaf {
            size: Size::default(),
            content_size: Size::default(),
            tabs,
            active: 0,
        }
    }

    pub fn set_size(&mut self, size: Size<Pixels>) {
        match self {
            Self::Empty => {}
            Self::Leaf { size: old_size, .. } => {
                *old_size = size;
            }
            Self::Vertical { size: old_size, .. } => {
                *old_size = size;
            }
            Self::Horizontal { size: old_size, .. } => {
                *old_size = size;
            }
        }
    }

    pub fn size(&self) -> Size<Pixels> {
        match self {
            Self::Empty => Size::default(),
            Self::Leaf { size, .. } => *size,
            Self::Vertical { size, .. } => *size,
            Self::Horizontal { size, .. } => *size,
        }
    }

    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    #[inline(always)]
    pub fn is_leaf(&self) -> bool {
        matches!(self, Self::Leaf { .. })
    }

    #[inline(always)]
    pub const fn is_horizontal(&self) -> bool {
        matches!(self, Self::Horizontal { .. })
    }

    #[inline(always)]
    pub const fn is_vertical(&self) -> bool {
        matches!(self, Self::Vertical { .. })
    }

    #[inline(always)]
    pub const fn is_parent(&self) -> bool {
        self.is_horizontal() || self.is_vertical()
    }

    /// Replaces the node with [`Horizontal`](Node::Horizontal) or [`Vertical`](Node::Vertical) (depending on `split`)
    /// and assigns an empty rect to it.
    ///
    /// # Panics
    ///
    /// If `fraction` isn't in range 0..=1.
    #[inline]
    pub fn split(&mut self, split: Split, fraction: f32) -> Self {
        assert!((0.0..=1.0).contains(&fraction));
        let size = Size::default();
        let src = match split {
            Split::Left | Split::Right => Node::Horizontal { fraction, size },
            Split::Above | Split::Below => Node::Vertical { fraction, size },
        };
        std::mem::replace(self, src)
    }

    pub fn tabs(&self) -> Option<&[Tab]> {
        match self {
            Self::Leaf { tabs, .. } => Some(tabs),
            _ => None,
        }
    }

    pub fn tabs_mut(&mut self) -> Option<&mut Vec<Tab>> {
        match self {
            Self::Leaf { tabs, .. } => Some(tabs),
            _ => None,
        }
    }

    pub fn iter_tabs(&self) -> impl Iterator<Item = &Tab> {
        self.tabs().into_iter().flat_map(|tabs| tabs.iter())
    }

    pub fn iter_tabs_mut(&mut self) -> impl Iterator<Item = &mut Tab> {
        self.tabs_mut().into_iter().flat_map(|tabs| tabs.iter_mut())
    }

    pub fn append_tab(&mut self, tab: Tab) {
        match self {
            Node::Leaf { tabs, active, .. } => {
                *active = tabs.len();
                tabs.push(tab);
            }
            _ => unreachable!(),
        }
    }

    pub fn insert_tab(&mut self, ix: usize, tab: Tab) {
        match self {
            Node::Leaf { tabs, active, .. } => {
                *active = ix;
                tabs.insert(ix, tab);
            }
            _ => unreachable!(),
        }
    }

    pub fn remove_tab(&mut self, ix: usize) -> Option<Tab> {
        match self {
            Node::Leaf { tabs, active, .. } => {
                if ix == *active {
                    *active = 0;
                }
                tabs.remove(ix).into()
            }
            _ => None,
        }
    }

    pub fn tabs_count(&self) -> usize {
        match self {
            Node::Leaf { tabs, .. } => tabs.len(),
            _ => 0,
        }
    }

    /// Returns a new [`Node`] while mapping and filtering the tab type.
    /// If this [`Node`] remains empty, it will change to [`Node::Empty`].
    pub fn filter_map_tabs<F, NewTab>(&self, function: F) -> Node<NewTab>
    where
        F: FnMut(&Tab) -> Option<NewTab>,
    {
        match self {
            Node::Leaf {
                size,
                content_size,
                tabs,
                active,
            } => {
                let tabs: Vec<_> = tabs.iter().filter_map(function).collect();
                if tabs.is_empty() {
                    Node::Empty
                } else {
                    Node::Leaf {
                        size: *size,
                        content_size: *content_size,
                        tabs,
                        active: *active,
                    }
                }
            }
            Node::Empty => Node::Empty,
            Node::Vertical { size, fraction } => Node::Vertical {
                size: *size,
                fraction: *fraction,
            },
            Node::Horizontal { size, fraction } => Node::Horizontal {
                size: *size,
                fraction: *fraction,
            },
        }
    }

    /// Returns a new [`Node`] while mapping the tab type.
    pub fn map_tabs<F, NewTab>(&self, mut function: F) -> Node<NewTab>
    where
        F: FnMut(&Tab) -> NewTab,
    {
        self.filter_map_tabs(move |tab| Some(function(tab)))
    }

    /// Returns a new [`Node`] while filtering the tab type.
    /// If this [`Node`] remains empty, it will change to [`Node::Empty`].
    pub fn filter_tabs<F>(&self, mut predicate: F) -> Node<Tab>
    where
        F: Clone + FnMut(&Tab) -> bool,
        Tab: Clone,
    {
        self.filter_map_tabs(move |tab| predicate(tab).then(|| tab.clone()))
    }

    /// Removes all tabs for which `predicate` returns `false`.
    /// If this [`Node`] remains empty, it will change to [`Node::Empty`].
    pub fn retain_tabs<F>(&mut self, predicate: F)
    where
        F: Clone + FnMut(&mut Tab) -> bool,
    {
        if let Node::Leaf { tabs, .. } = self {
            tabs.retain_mut(predicate);
            if tabs.is_empty() {
                *self = Node::Empty;
            }
        }
    }
}
