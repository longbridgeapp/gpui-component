mod node;
mod node_index;
mod tab_iter;

use std::{
    collections::HashSet,
    fmt,
    ops::{Index, IndexMut},
    slice::{Iter, IterMut},
};

use gpui::{Bounds, Pixels, Size};
use node::Node;
use node_index::NodeIndex;
use serde::{Deserialize, Serialize};
use tab_iter::TabIter;

/// The direction in which to split.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Split {
    Left,
    Right,
    Above,
    Below,
}

impl Split {
    pub fn is_horizontal(self) -> bool {
        matches!(self, Split::Left | Split::Right)
    }

    pub fn is_vertical(self) -> bool {
        matches!(self, Split::Above | Split::Below)
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
#[derive(Clone, Deserialize, Serialize)]
pub struct Tree<Tab> {
    // Binary tree vector
    pub(super) nodes: Vec<Node<Tab>>,
    focused_node: Option<NodeIndex>,
}

impl<Tab> fmt::Debug for Tree<Tab> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tree").finish_non_exhaustive()
    }
}

impl<Tab> Default for Tree<Tab> {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            focused_node: None,
        }
    }
}

impl<Tab> Index<NodeIndex> for Tree<Tab> {
    type Output = Node<Tab>;

    #[inline(always)]
    fn index(&self, index: NodeIndex) -> &Self::Output {
        &self.nodes[index.0]
    }
}

impl<Tab> IndexMut<NodeIndex> for Tree<Tab> {
    #[inline(always)]
    fn index_mut(&mut self, index: NodeIndex) -> &mut Self::Output {
        &mut self.nodes[index.0]
    }
}

impl<Tab> Tree<Tab> {
    /// Creates a new [`Tree`] with given `Vec` of `Tab`s in its root node.
    #[inline(always)]
    pub fn new(tabs: Vec<Tab>) -> Self {
        let root = Node::leaf_with(tabs);
        Self {
            nodes: vec![root],
            focused_node: None,
        }
    }

    /// Returns the viewport [`Rect`] and the `Tab` inside the first leaf node,
    /// or `None` if no leaf exists in the [`Tree`].
    #[inline]
    pub fn find_active(&mut self) -> Option<(Size<Pixels>, &mut Tab)> {
        self.nodes.iter_mut().find_map(|node| match node {
            Node::Leaf {
                tabs,
                active,
                content_size,
                ..
            } => tabs
                .get_mut(*active)
                .map(|tab| (content_size.to_owned(), tab)),
            _ => None,
        })
    }

    /// Returns the number of nodes in the [`Tree`].
    ///
    /// This includes [`Empty`](Node::Empty) nodes.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if the number of nodes in the tree is 0, otherwise `false`.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns an [`Iterator`] of the underlying collection of nodes.
    ///
    /// This includes [`Empty`](Node::Empty) nodes.
    #[inline(always)]
    pub fn iter(&self) -> Iter<'_, Node<Tab>> {
        self.nodes.iter()
    }

    /// Returns [`IterMut`] of the underlying collection of nodes.
    ///
    /// This includes [`Empty`](Node::Empty) nodes.
    #[inline(always)]
    pub fn iter_mut(&mut self) -> IterMut<'_, Node<Tab>> {
        self.nodes.iter_mut()
    }

    /// Returns an [`Iterator`] of [`NodeIndex`] ordered in a breadth first manner.
    #[inline(always)]
    pub(crate) fn breadth_first_index_iter(&self) -> impl Iterator<Item = NodeIndex> {
        (0..self.nodes.len()).map(NodeIndex)
    }

    /// Returns an iterator over all tabs in arbitrary order.
    #[inline(always)]
    pub fn tabs(&self) -> TabIter<'_, Tab> {
        TabIter::new(self)
    }

    /// Counts and returns the number of tabs in the whole tree.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use egui_dock::{DockState, NodeIndex, TabIndex};
    /// let mut dock_state = DockState::new(vec!["node 1", "node 2", "node 3"]);
    /// assert_eq!(dock_state.main_surface().num_tabs(), 3);
    ///
    /// let [a, b] = dock_state.main_surface_mut().split_left(NodeIndex::root(), 0.5, vec!["tab 4", "tab 5"]);
    /// assert_eq!(dock_state.main_surface().num_tabs(), 5);
    ///
    /// dock_state.main_surface_mut().remove_leaf(a);
    /// assert_eq!(dock_state.main_surface().num_tabs(), 2);
    /// ```
    #[inline]
    pub fn num_tabs(&self) -> usize {
        let mut count = 0;
        for node in self.nodes.iter() {
            if let Node::Leaf { tabs, .. } = node {
                count += tabs.len();
            }
        }
        count
    }

    /// Acquire a immutable borrow to the [`Node`] at the root of the tree.
    /// Returns [`None`] if the tree is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use egui_dock::DockState;
    /// let mut dock_state = DockState::new(vec!["single tab"]);
    /// let root_node = dock_state.main_surface().root_node().unwrap();
    ///
    /// assert_eq!(root_node.tabs(), Some(["single tab"].as_slice()));
    /// ```
    pub fn root_node(&self) -> Option<&Node<Tab>> {
        self.nodes.first()
    }

    /// Acquire a mutable borrow to the [`Node`] at the root of the tree.
    /// Returns [`None`] if the tree is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use egui_dock::{DockState, Node};
    /// let mut dock_state = DockState::new(vec!["single tab"]);
    /// let root_node = dock_state.main_surface_mut().root_node_mut().unwrap();
    /// if let Node::Leaf { tabs, ..} = root_node {
    ///     tabs.push("partner tab");
    /// }
    /// assert_eq!(root_node.tabs(), Some(["single tab", "partner tab"].as_slice()));
    /// ```
    pub fn root_node_mut(&mut self) -> Option<&mut Node<Tab>> {
        self.nodes.first_mut()
    }

    /// Creates two new nodes by splitting a given `parent` node and assigns them as its children. The first (old) node
    /// inherits content of the `parent` from before the split, and the second (new) gets the `tabs`.
    ///
    /// `fraction` (in range 0..=1) specifies how much of the `parent` node's area the old node will occupy after the
    /// split.
    ///
    /// The new node is placed relatively to the old node, in the direction specified by `split`.
    ///
    /// Returns the indices of the old node and the new node.
    ///
    /// # Panics
    ///
    /// If `fraction` isn't in range 0..=1.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use egui_dock::{DockState, SurfaceIndex, NodeIndex, Split};
    /// let mut dock_state = DockState::new(vec!["tab 1", "tab 2"]);
    ///
    /// // At this point, the main surface only contains the leaf with tab 1 and 2.
    /// assert!(dock_state.main_surface().root_node().unwrap().is_leaf());
    ///
    /// // Split the node, giving 50% of the space to the new nodes and 50% to the old ones.
    /// let [old, new] = dock_state.main_surface_mut()
    ///     .split_tabs(NodeIndex::root(), Split::Below, 0.5, vec!["tab 3"]);
    ///
    /// assert!(dock_state.main_surface().root_node().unwrap().is_parent());
    /// assert!(dock_state[SurfaceIndex::main()][old].is_leaf());
    /// assert!(dock_state[SurfaceIndex::main()][new].is_leaf());
    /// ```
    #[inline(always)]
    pub fn split_tabs(
        &mut self,
        parent: NodeIndex,
        split: Split,
        fraction: f32,
        tabs: Vec<Tab>,
    ) -> [NodeIndex; 2] {
        self.split(parent, split, fraction, Node::leaf_with(tabs))
    }

    /// Creates two new nodes by splitting a given `parent` node and assigns them as its children. The first (old) node
    /// inherits content of the `parent` from before the split, and the second (new) gets the `tabs`.
    ///
    /// This is a shorthand for using `split_tabs` with [`Split::Above`].
    ///
    /// `fraction` (in range 0..=1) specifies how much of the `parent` node's area the old node will occupy after the
    /// split.
    ///
    /// The new node is placed *above* the old node.
    ///
    /// Returns the indices of the old node and the new node.
    ///
    /// # Panics
    ///
    /// If `fraction` isn't in range 0..=1.
    #[inline(always)]
    pub fn split_above(
        &mut self,
        parent: NodeIndex,
        fraction: f32,
        tabs: Vec<Tab>,
    ) -> [NodeIndex; 2] {
        self.split(parent, Split::Above, fraction, Node::leaf_with(tabs))
    }

    /// Creates two new nodes by splitting a given `parent` node and assigns them as its children. The first (old) node
    /// inherits content of the `parent` from before the split, and the second (new) gets the `tabs`.
    ///
    /// This is a shorthand for using `split_tabs` with [`Split::Below`].
    ///
    /// `fraction` (in range 0..=1) specifies how much of the `parent` node's area the old node will occupy after the
    /// split.
    ///
    /// The new node is placed *below* the old node.
    ///
    /// Returns the indices of the old node and the new node.
    ///
    /// # Panics
    ///
    /// If `fraction` isn't in range 0..=1.
    #[inline(always)]
    pub fn split_below(
        &mut self,
        parent: NodeIndex,
        fraction: f32,
        tabs: Vec<Tab>,
    ) -> [NodeIndex; 2] {
        self.split(parent, Split::Below, fraction, Node::leaf_with(tabs))
    }

    /// Creates two new nodes by splitting a given `parent` node and assigns them as its children. The first (old) node
    /// inherits content of the `parent` from before the split, and the second (new) gets the `tabs`.
    ///
    /// This is a shorthand for using `split_tabs` with [`Split::Left`].
    ///
    /// `fraction` (in range 0..=1) specifies how much of the `parent` node's area the old node will occupy after the
    /// split.
    ///
    /// The new node is placed to the *left* of the old node.
    ///
    /// Returns the indices of the old node and the new node.
    ///
    /// # Panics
    ///
    /// If `fraction` isn't in range 0..=1.
    #[inline(always)]
    pub fn split_left(
        &mut self,
        parent: NodeIndex,
        fraction: f32,
        tabs: Vec<Tab>,
    ) -> [NodeIndex; 2] {
        self.split(parent, Split::Left, fraction, Node::leaf_with(tabs))
    }

    /// Creates two new nodes by splitting a given `parent` node and assigns them as its children. The first (old) node
    /// inherits content of the `parent` from before the split, and the second (new) gets the `tabs`.
    ///
    /// This is a shorthand for using `split_tabs` with [`Split::Right`].
    ///
    /// `fraction` (in range 0..=1) specifies how much of the `parent` node's area the old node will occupy after the
    /// split.
    ///
    /// The new node is placed to the *right* of the old node.
    ///
    /// Returns the indices of the old node and the new node.
    ///
    /// # Panics
    ///
    /// If `fraction` isn't in range 0..=1.
    #[inline(always)]
    pub fn split_right(
        &mut self,
        parent: NodeIndex,
        fraction: f32,
        tabs: Vec<Tab>,
    ) -> [NodeIndex; 2] {
        self.split(parent, Split::Right, fraction, Node::leaf_with(tabs))
    }

    /// Creates two new nodes by splitting a given `parent` node and assigns them as its children. The first (old) node
    /// inherits content of the `parent` from before the split, and the second (new) uses `new`.
    ///
    /// `fraction` (in range 0..=1) specifies how much of the `parent` node's area the old node will occupy after the
    /// split.
    ///
    /// The new node is placed relatively to the old node, in the direction specified by `split`.
    ///
    /// Returns the indices of the old node and the new node.
    ///
    /// # Panics
    ///
    /// If `fraction` isn't in range 0..=1.
    ///
    /// If `new` is an [`Empty`](Node::Empty), [`Horizontal`](Node::Horizontal) or [`Vertical`](Node::Vertical) node.
    ///
    /// If `new` is a [`Leaf`](Node::Leaf) node without any tabs.
    ///
    /// If `parent` points to an [`Empty`](Node::Empty) node.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use egui_dock::{DockState, SurfaceIndex, NodeIndex, Split, Node};
    /// let mut dock_state = DockState::new(vec!["tab 1", "tab 2"]);
    ///
    /// // At this point, the main surface only contains the leaf with tab 1 and 2.
    /// assert!(dock_state.main_surface().root_node().unwrap().is_leaf());
    ///
    /// // Splits the node, giving 50% of the space to the new nodes and 50% to the old ones.
    /// let [old, new] = dock_state.main_surface_mut()
    ///     .split(NodeIndex::root(), Split::Below, 0.5, Node::leaf_with(vec!["tab 3"]));
    ///
    /// assert!(dock_state.main_surface().root_node().unwrap().is_parent());
    /// assert!(dock_state[SurfaceIndex::main()][old].is_leaf());
    /// assert!(dock_state[SurfaceIndex::main()][new].is_leaf());
    /// ```
    pub fn split(
        &mut self,
        parent: NodeIndex,
        split: Split,
        fraction: f32,
        new: Node<Tab>,
    ) -> [NodeIndex; 2] {
        let old = self[parent].split(split, fraction);
        assert!(old.is_leaf() || old.is_parent());
        assert_ne!(new.tabs_count(), 0);
        // Resize vector to fit the new size of the binary tree.
        {
            let index = self.nodes.iter().rposition(|n| !n.is_empty()).unwrap_or(0);
            let level = NodeIndex(index).level();
            self.nodes
                .resize_with((1 << (level + 1)) - 1, || Node::Empty);
        }

        let index = match split {
            Split::Left | Split::Above => [parent.right(), parent.left()],
            Split::Right | Split::Below => [parent.left(), parent.right()],
        };

        // If the node were splitting is a parent, all it's children need to be moved.
        if old.is_parent() {
            let levels_to_move = NodeIndex(self.nodes.len()).level() - index[0].level();

            // Level 0 is ourself, which is done when we assign self[index[0]] = old, so start at 1.
            for level in (1..levels_to_move).rev() {
                // Old child indices for this level
                let old_start = parent.children_at(level).start;
                // New child indices for this level
                let new_start = index[0].children_at(level).start;

                // Children to be moved this level change
                let len = 1 << level;

                // Swap self[old_start..(old_start+len)] with self[new_start..(new_start+len)]
                // (the new part will only contain empty entries).
                let (old_range, new_range) = {
                    let (first_part, second_part) = self.nodes.split_at_mut(new_start);
                    // Cut to length.
                    (
                        &mut first_part[old_start..old_start + len],
                        &mut second_part[..len],
                    )
                };
                old_range.swap_with_slice(new_range);
            }
        }

        self[index[0]] = old;
        self[index[1]] = new;

        self.focused_node = Some(index[1]);

        index
    }

    fn first_leaf(&self, top: NodeIndex) -> Option<NodeIndex> {
        let left = top.left();
        let right = top.right();
        match (self.nodes.get(left.0), self.nodes.get(right.0)) {
            (Some(&Node::Leaf { .. }), _) => Some(left),
            (_, Some(&Node::Leaf { .. })) => Some(right),

            (
                Some(Node::Horizontal { .. } | Node::Vertical { .. }),
                Some(Node::Horizontal { .. } | Node::Vertical { .. }),
            ) => self.first_leaf(left).or(self.first_leaf(right)),
            (Some(Node::Horizontal { .. } | Node::Vertical { .. }), _) => self.first_leaf(left),
            (_, Some(Node::Horizontal { .. } | Node::Vertical { .. })) => self.first_leaf(right),

            (None, None)
            | (Some(&Node::Empty), None)
            | (None, Some(&Node::Empty))
            | (Some(&Node::Empty), Some(&Node::Empty)) => None,
        }
    }

    /// Returns the viewport [`Rect`] and the `Tab` inside the focused leaf node or [`None`] if it does not exist.
    #[inline]
    pub fn find_active_focused(&mut self) -> Option<(Size<Pixels>, &mut Tab)> {
        match self.focused_node.and_then(|idx| self.nodes.get_mut(idx.0)) {
            Some(Node::Leaf {
                tabs,
                active,
                content_size,
                ..
            }) => tabs.get_mut(*active).map(|tab| (*content_size, tab)),
            _ => None,
        }
    }

    /// Gets the node index of currently focused leaf node; returns [`None`] when no leaf is focused.
    #[inline]
    pub fn focused_leaf(&self) -> Option<NodeIndex> {
        self.focused_node
    }

    /// Sets the currently focused leaf to `node_index` if the node at `node_index` is a leaf.
    ///
    /// This method will not never panic and instead removes focus from all nodes when given an invalid index.
    #[inline]
    pub fn set_focused_node(&mut self, node_index: NodeIndex) {
        self.focused_node = self
            .nodes
            .get(node_index.0)
            .filter(|node| node.is_leaf())
            .map(|_| node_index);
    }

    /// Removes the given node from the [`Tree`].
    ///
    /// # Panics
    ///
    /// - If the tree is empty.
    /// - If the node at index `node` is not a [`Leaf`](Node::Leaf).
    pub fn remove_leaf(&mut self, node: NodeIndex) {
        assert!(!self.is_empty());
        assert!(self[node].is_leaf());

        let Some(parent) = node.parent() else {
            self.nodes.clear();
            return;
        };

        if Some(node) == self.focused_node {
            self.focused_node = None;
            let mut node = node;
            while let Some(parent) = node.parent() {
                let next = if node.is_left() {
                    parent.right()
                } else {
                    parent.left()
                };
                if self.nodes.get(next.0).is_some_and(|node| node.is_leaf()) {
                    self.focused_node = Some(next);
                    break;
                }
                if let Some(node) = self.first_leaf(next) {
                    self.focused_node = Some(node);
                    break;
                }
                node = parent;
            }
        }

        self[parent] = Node::Empty;
        self[node] = Node::Empty;

        let mut level = 0;

        if node.is_left() {
            'left_end: loop {
                let dst = parent.children_at(level);
                let src = parent.children_right(level + 1);
                for (dst, src) in dst.zip(src) {
                    if src >= self.nodes.len() {
                        break 'left_end;
                    }
                    if Some(NodeIndex(src)) == self.focused_node {
                        self.focused_node = Some(NodeIndex(dst));
                    }
                    self.nodes[dst] = std::mem::replace(&mut self.nodes[src], Node::Empty);
                }
                level += 1;
            }
        } else {
            'right_end: loop {
                let dst = parent.children_at(level);
                let src = parent.children_left(level + 1);
                for (dst, src) in dst.zip(src) {
                    if src >= self.nodes.len() {
                        break 'right_end;
                    }
                    if Some(NodeIndex(src)) == self.focused_node {
                        self.focused_node = Some(NodeIndex(dst));
                    }
                    self.nodes[dst] = std::mem::replace(&mut self.nodes[src], Node::Empty);
                }
                level += 1;
            }
        }
    }

    /// Pushes a tab to the first `Leaf` it finds or create a new leaf if an `Empty` node is encountered.
    pub fn push_to_first_leaf(&mut self, tab: Tab) {
        for (index, node) in &mut self.nodes.iter_mut().enumerate() {
            match node {
                Node::Leaf { tabs, active, .. } => {
                    *active = tabs.len();
                    tabs.push(tab);
                    self.focused_node = Some(NodeIndex(index));
                    return;
                }
                Node::Empty => {
                    *node = Node::leaf(tab);
                    self.focused_node = Some(NodeIndex(index));
                    return;
                }
                _ => {}
            }
        }
        assert!(self.nodes.is_empty());
        self.nodes.push(Node::leaf_with(vec![tab]));
        self.focused_node = Some(NodeIndex(0));
    }

    /// Sets which is the active tab within a specific node.
    #[inline]
    pub fn set_active_tab(&mut self, node_index: NodeIndex, tab_ix: usize) {
        if let Some(Node::Leaf { active, .. }) = self.nodes.get_mut(node_index.0) {
            *active = tab_ix;
        }
    }

    /// Pushes `tab` to the currently focused leaf.
    ///
    /// If no leaf is focused it will be pushed to the first available leaf.
    ///
    /// If no leaf is available then a new leaf will be created.
    pub fn push_to_focused_leaf(&mut self, tab: Tab) {
        match self.focused_node {
            Some(node) => {
                if self.nodes.is_empty() {
                    self.nodes.push(Node::leaf(tab));
                    self.focused_node = Some(NodeIndex::root());
                } else {
                    match &mut self[node] {
                        Node::Empty => {
                            self[node] = Node::leaf(tab);
                            self.focused_node = Some(node);
                        }
                        Node::Leaf { tabs, active, .. } => {
                            *active = tabs.len();
                            tabs.push(tab);
                            self.focused_node = Some(node);
                        }
                        _ => {
                            self.push_to_first_leaf(tab);
                        }
                    }
                }
            }
            None => {
                if self.nodes.is_empty() {
                    self.nodes.push(Node::leaf(tab));
                    self.focused_node = Some(NodeIndex::root());
                } else {
                    self.push_to_first_leaf(tab);
                }
            }
        }
    }

    /// Removes the tab at the given ([`NodeIndex`], [`TabIndex`]) pair.
    ///
    /// If the node is emptied after the tab is removed, the node will also be removed.
    ///
    /// Returns the removed tab if it exists, or `None` otherwise.
    pub fn remove_tab(&mut self, (node_ix, tab_ix): (NodeIndex, usize)) -> Option<Tab> {
        let node = &mut self[node_ix];
        let tab = node.remove_tab(tab_ix);
        if node.tabs_count() == 0 {
            self.remove_leaf(node_ix);
        }
        tab
    }

    /// Returns a new [`Tree`] while mapping and filtering the tab type.
    /// Any remaining empty [`Node`]s are removed.
    pub fn filter_map_tabs<F, NewTab>(&self, function: F) -> Tree<NewTab>
    where
        F: Clone + FnMut(&Tab) -> Option<NewTab>,
    {
        let Tree {
            focused_node,
            nodes,
        } = self;
        let mut emptied_nodes = HashSet::default();
        let nodes = nodes
            .iter()
            .enumerate()
            .map(|(index, node)| {
                let filtered_node = node.filter_map_tabs(function.clone());
                if filtered_node.is_empty() && !node.is_empty() {
                    emptied_nodes.insert(NodeIndex(index));
                }
                filtered_node
            })
            .collect();
        let mut new_tree = Tree {
            nodes,
            focused_node: *focused_node,
        };
        new_tree.balance(emptied_nodes);
        new_tree
    }

    /// Returns a new [`Tree`] while mapping the tab type.
    pub fn map_tabs<F, NewTab>(&self, mut function: F) -> Tree<NewTab>
    where
        F: Clone + FnMut(&Tab) -> NewTab,
    {
        self.filter_map_tabs(move |tab| Some(function(tab)))
    }

    /// Returns a new [`Tree`] while filtering the tab type.
    /// Any remaining empty [`Node`]s are removed.
    pub fn filter_tabs<F>(&self, mut predicate: F) -> Tree<Tab>
    where
        F: Clone + FnMut(&Tab) -> bool,
        Tab: Clone,
    {
        self.filter_map_tabs(move |tab| predicate(tab).then(|| tab.clone()))
    }

    /// Removes all tabs for which `predicate` returns `false`.
    /// Any remaining empty [`Node`]s are also removed.
    pub fn retain_tabs<F>(&mut self, predicate: F)
    where
        F: Clone + FnMut(&mut Tab) -> bool,
    {
        let mut emptied_nodes = HashSet::default();
        for (index, node) in self.nodes.iter_mut().enumerate() {
            node.retain_tabs(predicate.clone());
            if node.is_empty() {
                emptied_nodes.insert(NodeIndex(index));
            }
        }
        self.balance(emptied_nodes);
    }

    fn balance(&mut self, emptied_nodes: HashSet<NodeIndex>) {
        let mut emptied_parents = HashSet::default();
        for parent_index in emptied_nodes.into_iter().filter_map(|ni| ni.parent()) {
            if self[parent_index.left()].is_empty() && self[parent_index.right()].is_empty() {
                self[parent_index] = Node::Empty;
                emptied_parents.insert(parent_index);
            } else if self[parent_index.left()].is_empty() {
                self.nodes.swap(parent_index.0, parent_index.right().0);
                self[parent_index.right()] = Node::Empty;
            } else if self[parent_index.right()].is_empty() {
                self.nodes.swap(parent_index.0, parent_index.left().0);
                self[parent_index.left()] = Node::Empty;
            }
        }
        if !emptied_parents.is_empty() {
            self.balance(emptied_parents);
        }
    }
}

impl<Tab> Tree<Tab>
where
    Tab: PartialEq,
{
    /// Find the given tab.
    ///
    /// Returns in which node and where in that node the tab is.
    ///
    /// The returned [`NodeIndex`] will always point to a [`Node::Leaf`].
    ///
    /// In case there are several hits, only the first is returned.
    pub fn find_tab(&self, needle_tab: &Tab) -> Option<(NodeIndex, usize)> {
        for (node_index, node) in self.nodes.iter().enumerate() {
            if let Some(tabs) = node.tabs() {
                for (tab_ix, tab) in tabs.iter().enumerate() {
                    if tab == needle_tab {
                        return Some((node_index.into(), tab_ix));
                    }
                }
            }
        }
        None
    }
}
