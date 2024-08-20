mod node;

use gpui::{Bounds, Pixels};

/// The direction in which to split.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Split {
    Left,
    Right,
    Top,
    Bottom,
}

impl Split {
    pub fn is_horizontal(self) -> bool {
        matches!(self, Split::Left | Split::Right)
    }

    pub fn is_vertical(self) -> bool {
        matches!(self, Split::Top | Split::Bottom)
    }
}

/// Specifies how to insert a new tab.
pub enum TabInsert {
    Split(Split),
    Insert(usize),
    Append,
}

/// The destination of a tab which is being moved.
pub enum TabDestination {
    /// Move to new window with this bounds.
    Window(Bounds<Pixels>),
    /// Move to an existing node with this insert.
    Node(usize, usize, TabInsert),
    /// Move to an empty panel.
    EmptyPanel(usize),
}

impl TabDestination {
    pub fn is_window(&self) -> bool {
        matches!(self, TabDestination::Window(_))
    }
}

/// Binary tree representing the relationships between [`Node`]s.
///
/// # Implementation details
///
/// The binary tree is stored in a [`Vec`] indexed by [`NodeIndex`].
/// The root is always at index *0*.
/// For a given node *n*:
///  - left child of *n* will be at index *n * 2 + 1*.
///  - right child of *n* will be at index *n * 2 + 2*.
///
/// For "Horizontal" nodes:
///  - left child contains Left node.
///  - right child contains Right node.
///
/// For "Vertical" nodes:
///  - left child contains Top node.
///  - right child contains Bottom node.
#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Tree<Tab> {
    /// Binary tree vector
    pub(super) nodes: Vec<Node<Tab>>,
    focused_node: Option<NodeIndex>,
}
