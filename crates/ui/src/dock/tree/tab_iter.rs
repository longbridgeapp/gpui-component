use super::Tree;

/// Iterates over all tabs in a [`Tree`].
pub struct TabIter<'a, Tab> {
    tree: &'a Tree<Tab>,
    node_idx: usize,
    tab_idx: usize,
}

impl<'a, Tab> TabIter<'a, Tab> {
    pub(super) fn new(tree: &'a Tree<Tab>) -> Self {
        Self {
            tree,
            node_idx: 0,
            tab_idx: 0,
        }
    }
}

impl<'a, Tab> Iterator for TabIter<'a, Tab> {
    type Item = &'a Tab;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.tree.nodes.get(self.node_idx)?.tabs() {
                Some(tabs) => match tabs.get(self.tab_idx) {
                    Some(tab) => {
                        self.tab_idx += 1;
                        return Some(tab);
                    }
                    None => {
                        self.node_idx += 1;
                        self.tab_idx = 0;
                    }
                },
                None => {
                    self.node_idx += 1;
                    self.tab_idx = 0;
                }
            }
        }
    }
}

impl<'a, Tab> std::fmt::Debug for TabIter<'a, Tab> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TabIter").finish_non_exhaustive()
    }
}

#[test]
fn test_tabs_iter() {
    fn tabs(tree: &Tree<i32>) -> Vec<i32> {
        tree.tabs().copied().collect()
    }

    let mut tree = Tree::new(vec![1, 2, 3]);
    assert_eq!(tabs(&tree), vec![1, 2, 3]);

    tree.push_to_first_leaf(4);
    assert_eq!(tabs(&tree), vec![1, 2, 3, 4]);

    tree.push_to_first_leaf(5);
    assert_eq!(tabs(&tree), vec![1, 2, 3, 4, 5]);

    tree.push_to_focused_leaf(6);
    assert_eq!(tabs(&tree), vec![1, 2, 3, 4, 5, 6]);

    assert_eq!(tree.num_tabs(), tree.tabs().count());
}
