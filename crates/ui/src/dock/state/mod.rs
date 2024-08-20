use gpui::{Bounds, Pixels};

mod surface;
mod surface_index;
mod tree;
mod window_state;

pub use surface::Surface;
pub use surface_index::SurfaceIndex;
pub use tree::{Node, NodeIndex, Split, TabDestination, TabInsert, Tree};
pub use window_state::WindowState;

/// This structure holds a collection of surfaces, each of which stores a tree in which tabs are arranged.
///
/// Indexing it with a [`SurfaceIndex`] will yield a [`Tree`] which then contains nodes and tabs.
///
/// [`DockState`] is generic, so you can use any type of data to represent a tab.
#[derive(Clone, Debug)]
pub struct DockState<Tab> {
    surfaces: Vec<Surface<Tab>>,
    // Part of the tree which is in focus.
    focused_surface: Option<SurfaceIndex>,
}

impl<Tab> std::ops::Index<SurfaceIndex> for DockState<Tab> {
    type Output = Tree<Tab>;

    #[inline(always)]
    fn index(&self, index: SurfaceIndex) -> &Self::Output {
        match self.surfaces[index.0].node_tree() {
            Some(tree) => tree,
            None => {
                panic!("There did not exist a tree at surface index {}", index.0);
            }
        }
    }
}

impl<Tab> std::ops::IndexMut<SurfaceIndex> for DockState<Tab> {
    #[inline(always)]
    fn index_mut(&mut self, index: SurfaceIndex) -> &mut Self::Output {
        match self.surfaces[index.0].node_tree_mut() {
            Some(tree) => tree,
            None => {
                panic!("There did not exist a tree at surface index {}", index.0);
            }
        }
    }
}

impl<Tab> DockState<Tab> {
    /// Create a new tree with given tabs at the main surface's root node.
    pub fn new(tabs: Vec<Tab>) -> Self {
        Self {
            surfaces: vec![Surface::Main(Tree::new(tabs))],
            focused_surface: None,
        }
    }

    /// Get an immutable borrow to the tree at the main surface.
    pub fn main_surface(&self) -> &Tree<Tab> {
        &self[SurfaceIndex::main()]
    }

    /// Get a mutable borrow to the tree at the main surface.
    pub fn main_surface_mut(&mut self) -> &mut Tree<Tab> {
        &mut self[SurfaceIndex::main()]
    }

    /// Get the [`WindowState`] which corresponds to a [`SurfaceIndex`].
    ///
    /// Returns `None` if the surface is [`Empty`](Surface::Empty), [`Main`](Surface::Main), or doesn't exist.
    ///
    /// This can be used to modify properties of a window, e.g. size and position.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use egui_dock::DockState;
    /// # use egui::{Vec2, Pos2};
    /// let mut dock_state = DockState::new(vec![]);
    /// let mut surface_index = dock_state.add_window(vec!["Window Tab".to_string()]);
    /// let window_state = dock_state.get_window_state_mut(surface_index).unwrap();
    ///
    /// window_state.set_position(Pos2::ZERO);
    /// window_state.set_size(Vec2::splat(100.0));
    /// ```
    pub fn get_window_state_mut(&mut self, surface: SurfaceIndex) -> Option<&mut WindowState> {
        match &mut self.surfaces[surface.0] {
            Surface::Window(_, state) => Some(state),
            _ => None,
        }
    }

    /// Get the [`WindowState`] which corresponds to a [`SurfaceIndex`].
    ///
    /// Returns `None` if the surface is an [`Empty`](Surface::Empty), [`Main`](Surface::Main), or doesn't exist.
    pub fn get_window_state(&mut self, surface: SurfaceIndex) -> Option<&WindowState> {
        match &self.surfaces[surface.0] {
            Surface::Window(_, state) => Some(state),
            _ => None,
        }
    }

    /// Returns the viewport [`Rect`] and the `Tab` inside the focused leaf node or `None` if no node is in focus.
    #[inline]
    pub fn find_active_focused(&mut self) -> Option<(Bounds<Pixels>, &mut Tab)> {
        self.focused_surface
            .and_then(|surface| self[surface].find_active_focused())
    }

    /// Get a mutable borrow to the raw surface from a surface index.
    #[inline]
    pub fn get_surface_mut(&mut self, surface: SurfaceIndex) -> Option<&mut Surface<Tab>> {
        self.surfaces.get_mut(surface.0)
    }

    /// Get an immutable borrow to the raw surface from a surface index.
    #[inline]
    pub fn get_surface(&self, surface: SurfaceIndex) -> Option<&Surface<Tab>> {
        self.surfaces.get(surface.0)
    }

    /// Returns true if the specified surface exists and isn't [`Empty`](Surface::Empty).
    #[inline]
    pub fn is_surface_valid(&self, surface_index: SurfaceIndex) -> bool {
        self.surfaces
            .get(surface_index.0)
            .map_or(false, |surface| !surface.is_empty())
    }

    /// Returns a list of all valid [`SurfaceIndex`]es.
    #[inline]
    pub(crate) fn valid_surface_indices(&self) -> Box<[SurfaceIndex]> {
        (0..self.surfaces.len())
            .filter_map(|index| {
                let index = SurfaceIndex(index);
                self.is_surface_valid(index).then_some(index)
            })
            .collect()
    }

    /// Remove a surface based on its [`SurfaceIndex`]
    ///
    /// Returns the removed surface or `None` if it didn't exist.
    ///
    /// # Panics
    ///
    /// Panics if you try to remove the main surface: `SurfaceIndex::main()`.
    pub fn remove_surface(&mut self, surface_index: SurfaceIndex) -> Option<Surface<Tab>> {
        assert!(!surface_index.is_main());
        (surface_index.0 < self.surfaces.len()).then(|| {
            self.focused_surface = Some(SurfaceIndex::main());
            if surface_index.0 == self.surfaces.len() - 1 {
                self.surfaces.pop().unwrap()
            } else {
                let dest = &mut self.surfaces[surface_index.0];
                std::mem::replace(dest, Surface::Empty)
            }
        })
    }

    /// Sets which is the active tab within a specific node on a given surface.
    #[inline]
    pub fn set_active_tab(
        &mut self,
        (surface_index, node_index, tab_index): (SurfaceIndex, NodeIndex, usize),
    ) {
        if let Some(Node::Leaf { active, .. }) = self[surface_index].nodes.get_mut(node_index.0) {
            *active = tab_index;
        }
    }

    /// Sets the currently focused leaf to `node_index` if the node at `node_index` is a leaf.
    #[inline]
    pub fn set_focused_node_and_surface(
        &mut self,
        (surface_index, node_index): (SurfaceIndex, NodeIndex),
    ) {
        if self.is_surface_valid(surface_index) && node_index.0 < self[surface_index].len() {
            // I don't want this code to be evaluated until im absolutely sure the surface index is valid.
            if self[surface_index][node_index].is_leaf() {
                self.focused_surface = Some(surface_index);
                self[surface_index].set_focused_node(node_index);
                return;
            }
        }
        self.focused_surface = None;
    }

    /// Moves a tab from a node to another node.
    /// You need to specify with [`TabDestination`] how the tab should be moved.
    pub fn move_tab(
        &mut self,
        (src_surface, src_node, src_tab): (SurfaceIndex, NodeIndex, usize),
        dst_tab: impl Into<TabDestination>,
    ) {
        match dst_tab.into() {
            TabDestination::Window(position) => {
                self.detach_tab((src_surface, src_node, src_tab), position);
                return;
            }
            TabDestination::Node(dst_surface, dst_node, dst_tab) => {
                // Moving a single tab inside its own node is a no-op
                if src_surface == dst_surface
                    && src_node == dst_node
                    && self[src_surface][src_node].tabs_count() == 1
                {
                    return;
                }

                // Call `Node::remove_tab` to avoid auto remove of the node by `Tree::remove_tab` from Tree.
                let tab = self[src_surface][src_node].remove_tab(src_tab).unwrap();
                match dst_tab {
                    TabInsert::Split(split) => {
                        self[dst_surface].split(dst_node, split, 0.5, Node::leaf(tab));
                    }

                    TabInsert::Insert(index) => self[dst_surface][dst_node].insert_tab(index, tab),
                    TabInsert::Append => self[dst_surface][dst_node].append_tab(tab),
                }
            }
            TabDestination::EmptySurface(dst_surface) => {
                assert!(self[dst_surface].is_empty());
                let tab = self[src_surface][src_node].remove_tab(src_tab).unwrap();
                self[dst_surface] = Tree::new(vec![tab])
            }
        }
        if self[src_surface][src_node].is_leaf() && self[src_surface][src_node].tabs_count() == 0 {
            self[src_surface].remove_leaf(src_node);
        }
        if self[src_surface].is_empty() && !src_surface.is_main() {
            self.remove_surface(src_surface);
        }
    }

    /// Takes a tab out of its current surface and puts it in a new window.
    /// Returns the surface index of the new window.
    pub fn detach_tab(
        &mut self,
        (src_surface, src_node, src_tab): (SurfaceIndex, NodeIndex, usize),
        window_bounds: Bounds<Pixels>,
    ) -> SurfaceIndex {
        // Remove the tab from the tree and it add to a new window.
        let tab = self[src_surface][src_node].remove_tab(src_tab).unwrap();
        let surface_index = self.add_window(vec![tab]);

        // Set the window size and position to match `window_rect`.
        let state = self.get_window_state_mut(surface_index).unwrap();
        state.set_bounds(window_bounds);
        // if src_surface.is_main() {
        //     state.set_size(window_rect.size() * 0.8);
        // } else {
        //     state.set_size(window_rect.size());
        // }

        // Clean up any empty leaves and surfaces which may be left behind from the detachment.
        if self[src_surface][src_node].is_leaf() && self[src_surface][src_node].tabs_count() == 0 {
            self[src_surface].remove_leaf(src_node);
        }
        if self[src_surface].is_empty() && !src_surface.is_main() {
            self.remove_surface(src_surface);
        }
        surface_index
    }

    /// Currently focused leaf.
    #[inline]
    pub fn focused_leaf(&self) -> Option<(SurfaceIndex, NodeIndex)> {
        let surface = self.focused_surface?;
        self[surface].focused_leaf().map(|leaf| (surface, leaf))
    }

    /// Remove a tab at the specified surface, node, and tab index.
    /// This method will yield the removed tab, or `None` if it doesn't exist.
    pub fn remove_tab(
        &mut self,
        (surface_index, node_index, tab_index): (SurfaceIndex, NodeIndex, usize),
    ) -> Option<Tab> {
        let removed_tab = self[surface_index].remove_tab((node_index, tab_index));
        if !surface_index.is_main() && self[surface_index].is_empty() {
            self.remove_surface(surface_index);
        }
        removed_tab
    }

    /// Creates two new nodes by splitting a given `parent` node and assigns them as its children. The first (old) node
    /// inherits content of the `parent` from before the split, and the second (new) has `tabs`.
    ///
    /// `fraction` (in range 0..=1) specifies how much of the `parent` node's area the old node will occupy after the
    /// split.
    ///
    /// The new node is placed relatively to the old node, in the direction specified by `split`.
    ///
    /// Returns the indices of the old node and the new node.
    pub fn split(
        &mut self,
        (surface, parent): (SurfaceIndex, NodeIndex),
        split: Split,
        fraction: f32,
        new: Node<Tab>,
    ) -> [NodeIndex; 2] {
        let index = self[surface].split(parent, split, fraction, new);
        self.focused_surface = Some(surface);
        index
    }

    /// Adds a window with its own list of tabs.
    ///
    /// Returns the [`SurfaceIndex`] of the new window, which will remain constant through the windows lifetime.
    pub fn add_window(&mut self, tabs: Vec<Tab>) -> SurfaceIndex {
        let surface = Surface::Window(Tree::new(tabs), WindowState::new());
        let index = self.find_empty_surface_index();
        if index.0 < self.surfaces.len() {
            self.surfaces[index.0] = surface;
        } else {
            self.surfaces.push(surface);
        }
        index
    }

    /// Finds the first empty surface index which may be used.
    ///
    /// **WARNING**: in cases where one isn't found, `SurfaceIndex(self.surfaces.len())` is used.
    /// therefore it's not inherently safe to index the [`DockState`] with this index, as it may panic.
    fn find_empty_surface_index(&self) -> SurfaceIndex {
        // Find the first possible empty surface to insert our window into.
        // Starts at 1 as 0 is always the main surface.
        for i in 1..self.surfaces.len() {
            if self.surfaces[i].is_empty() {
                return SurfaceIndex(i);
            }
        }
        SurfaceIndex(self.surfaces.len())
    }

    /// Pushes `tab` to the currently focused leaf.
    ///
    /// If no leaf is focused it will be pushed to the first available leaf.
    ///
    /// If no leaf is available then a new leaf will be created.
    pub fn push_to_focused_leaf(&mut self, tab: Tab) {
        if let Some(surface) = self.focused_surface {
            self[surface].push_to_focused_leaf(tab)
        } else {
            self[SurfaceIndex::main()].push_to_focused_leaf(tab)
        }
    }

    /// Push a tab to the first available `Leaf` or create a new leaf if an `Empty` node is encountered.
    pub fn push_to_first_leaf(&mut self, tab: Tab) {
        self[SurfaceIndex::main()].push_to_first_leaf(tab);
    }

    /// Returns the current number of surfaces.
    pub fn surfaces_count(&self) -> usize {
        self.surfaces.len()
    }

    /// Returns an [`Iterator`] over all surfaces.
    pub fn iter_surfaces(&self) -> impl Iterator<Item = &Surface<Tab>> {
        self.surfaces.iter()
    }

    /// Returns a mutable [`Iterator`] over all surfaces.
    pub fn iter_surfaces_mut(&mut self) -> impl Iterator<Item = &mut Surface<Tab>> {
        self.surfaces.iter_mut()
    }

    /// Returns an [`Iterator`] of **all** underlying nodes in the dock state,
    /// and the indices of containing surfaces.
    pub fn iter_all_nodes(&self) -> impl Iterator<Item = (SurfaceIndex, &Node<Tab>)> {
        self.iter_surfaces()
            .enumerate()
            .flat_map(|(surface_index, surface)| {
                surface
                    .iter_nodes()
                    .map(move |node| (SurfaceIndex(surface_index), node))
            })
    }

    /// Returns a mutable [`Iterator`] of **all** underlying nodes in the dock state,
    /// and the indices of containing surfaces.
    pub fn iter_all_nodes_mut(&mut self) -> impl Iterator<Item = (SurfaceIndex, &mut Node<Tab>)> {
        self.iter_surfaces_mut()
            .enumerate()
            .flat_map(|(surface_index, surface)| {
                surface
                    .iter_nodes_mut()
                    .map(move |node| (SurfaceIndex(surface_index), node))
            })
    }

    /// Returns an [`Iterator`] of **all** tabs in the dock state,
    /// and the indices of containing surfaces and nodes.
    pub fn iter_all_tabs(&self) -> impl Iterator<Item = ((SurfaceIndex, NodeIndex), &Tab)> {
        self.iter_surfaces()
            .enumerate()
            .flat_map(|(surface_index, surface)| {
                surface
                    .iter_all_tabs()
                    .map(move |(node_index, tab)| ((SurfaceIndex(surface_index), node_index), tab))
            })
    }

    /// Returns a mutable [`Iterator`] of **all** tabs in the dock state,
    /// and the indices of containing surfaces and nodes.
    pub fn iter_all_tabs_mut(
        &mut self,
    ) -> impl Iterator<Item = ((SurfaceIndex, NodeIndex), &mut Tab)> {
        self.iter_surfaces_mut()
            .enumerate()
            .flat_map(|(surface_index, surface)| {
                surface
                    .iter_all_tabs_mut()
                    .map(move |(node_index, tab)| ((SurfaceIndex(surface_index), node_index), tab))
            })
    }

    /// Returns an [`Iterator`] of the underlying collection of nodes on the main surface.
    #[deprecated = "Use `dock_state.main_surface().iter()` instead"]
    pub fn iter_main_surface_nodes(&self) -> impl Iterator<Item = &Node<Tab>> {
        self[SurfaceIndex::main()].iter()
    }

    /// Returns a mutable [`Iterator`] of the underlying collection of nodes on the main surface.
    #[deprecated = "Use `dock_state.main_surface_mut().iter_mut()` instead"]
    pub fn iter_main_surface_nodes_mut(&mut self) -> impl Iterator<Item = &mut Node<Tab>> {
        self[SurfaceIndex::main()].iter_mut()
    }

    /// Returns an [`Iterator`] of **all** underlying nodes in the dock state and all subsequent trees.
    #[deprecated = "Use `iter_all_nodes` instead"]
    pub fn iter_nodes(&self) -> impl Iterator<Item = &Node<Tab>> {
        self.surfaces
            .iter()
            .filter_map(|surface| surface.node_tree())
            .flat_map(|nodes| nodes.iter())
    }

    /// Returns a new [`DockState`] while mapping and filtering the tab type.
    /// Any remaining empty [`Node`]s and [`Surface`]s are removed.
    ///
    /// ```
    /// # use egui_dock::{DockState, Node};
    /// let dock_state = DockState::new(vec![1, 2, 3]);
    /// let mapped_dock_state = dock_state.filter_map_tabs(|tab| (tab % 2 == 1).then(|| tab.to_string()));
    ///
    /// let tabs: Vec<_> = mapped_dock_state.iter_all_tabs().map(|(_, tab)| tab.to_owned()).collect();
    /// assert_eq!(tabs, vec!["1".to_string(), "3".to_string()]);
    /// ```
    pub fn filter_map_tabs<F, NewTab>(&self, function: F) -> DockState<NewTab>
    where
        F: Clone + FnMut(&Tab) -> Option<NewTab>,
    {
        let DockState {
            surfaces,
            focused_surface,
        } = self;
        let surfaces = surfaces
            .iter()
            .filter_map(|surface| {
                let surface = surface.filter_map_tabs(function.clone());
                (!surface.is_empty()).then_some(surface)
            })
            .collect();
        DockState {
            surfaces,
            focused_surface: *focused_surface,
        }
    }

    /// Returns a new [`DockState`] while mapping the tab type.
    ///
    /// ```
    /// # use egui_dock::{DockState, Node};
    /// let dock_state = DockState::new(vec![1, 2, 3]);
    /// let mapped_dock_state = dock_state.map_tabs(|tab| tab.to_string());
    ///
    /// let tabs: Vec<_> = mapped_dock_state.iter_all_tabs().map(|(_, tab)| tab.to_owned()).collect();
    /// assert_eq!(tabs, vec!["1".to_string(), "2".to_string(), "3".to_string()]);
    /// ```
    pub fn map_tabs<F, NewTab>(&self, mut function: F) -> DockState<NewTab>
    where
        F: Clone + FnMut(&Tab) -> NewTab,
    {
        self.filter_map_tabs(move |tab| Some(function(tab)))
    }

    /// Returns a new [`DockState`] while filtering the tab type.
    /// Any remaining empty [`Node`]s and [`Surface`]s are removed.
    ///
    /// ```
    /// # use egui_dock::{DockState, Node};
    /// let dock_state = DockState::new(["tab1", "tab2", "outlier"].map(str::to_string).to_vec());
    /// let filtered_dock_state = dock_state.filter_tabs(|tab| tab.starts_with("tab"));
    ///
    /// let tabs: Vec<_> = filtered_dock_state.iter_all_tabs().map(|(_, tab)| tab.to_owned()).collect();
    /// assert_eq!(tabs, vec!["tab1".to_string(), "tab2".to_string()]);
    /// ```
    pub fn filter_tabs<F>(&self, mut predicate: F) -> DockState<Tab>
    where
        F: Clone + FnMut(&Tab) -> bool,
        Tab: Clone,
    {
        self.filter_map_tabs(move |tab| predicate(tab).then(|| tab.clone()))
    }

    /// Removes all tabs for which `predicate` returns `false`.
    /// Any remaining empty [`Node`]s and [`Surface`]s are also removed.
    ///
    /// ```
    /// # use egui_dock::{DockState, Node};
    /// let mut dock_state = DockState::new(["tab1", "tab2", "outlier"].map(str::to_string).to_vec());
    /// dock_state.retain_tabs(|tab| tab.starts_with("tab"));
    ///
    /// let tabs: Vec<_> = dock_state.iter_all_tabs().map(|(_, tab)| tab.to_owned()).collect();
    /// assert_eq!(tabs, vec!["tab1".to_string(), "tab2".to_string()]);
    /// ```
    pub fn retain_tabs<F>(&mut self, predicate: F)
    where
        F: Clone + FnMut(&mut Tab) -> bool,
    {
        self.surfaces.retain_mut(|surface| {
            surface.retain_tabs(predicate.clone());
            !surface.is_empty()
        });
    }
}

impl<Tab> DockState<Tab>
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
    ///
    /// See also: [`find_main_surface_tab`](DockState::find_main_surface_tab)
    pub fn find_tab(&self, needle_tab: &Tab) -> Option<(SurfaceIndex, NodeIndex, usize)> {
        for &surface_index in self.valid_surface_indices().iter() {
            if !self.surfaces[surface_index.0].is_empty() {
                if let Some((node_index, tab_index)) = self[surface_index].find_tab(needle_tab) {
                    return Some((surface_index, node_index, tab_index));
                }
            }
        }
        None
    }

    /// Find the given tab on the main surface.
    ///
    /// Returns which node and where in that node the tab is.
    ///
    /// The returned [`NodeIndex`] will always point to a [`Node::Leaf`].
    ///
    /// In case there are several hits, only the first is returned.
    pub fn find_main_surface_tab(&self, needle_tab: &Tab) -> Option<(NodeIndex, usize)> {
        self[SurfaceIndex::main()].find_tab(needle_tab)
    }
}
