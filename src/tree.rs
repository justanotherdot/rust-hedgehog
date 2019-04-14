use lazy::Thunk;

#[allow(dead_code)]
pub struct Tree<'a, T> {
    thunk: Thunk<'a, T>,
    children: Vec<Tree<'a, T>>,
}

impl<'a, T: 'a + Clone> Tree<'a, T> {
    pub fn singleton(value: T) -> Tree<'a, T> {
        Tree {
            thunk: Thunk::new(move || value.clone()),
            children: vec![],
        }
    }

    pub fn value(&mut self) -> &T {
        self.thunk.force().value()
    }
}

#[cfg(test)]
mod tests {
    use tree::Tree;

    #[test]
    fn rose_trees_hold_lazy_values() {
        let n = 42;
        let mut tree = Tree::singleton(n);
        tree.value();
        tree.value();
        assert_eq!(*tree.value(), n);
    }
}
