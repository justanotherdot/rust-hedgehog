use lazy::Lazy;

pub struct Tree<'a, A> {
    thunk: Lazy<'a, A>,
    children: Vec<Tree<'a, A>>,
}

impl<'a, A: 'a + Clone> Tree<'a, A> {
    pub fn singleton(value: A) -> Tree<'a, A> {
        Tree {
            thunk: Lazy::new(move || value.clone()),
            children: vec![],
        }
    }

    pub fn value(&mut self) -> &A {
        self.thunk.value()
    }
}

pub fn render<'a, A>(root: &mut Tree<'a, A>)
where
    A: Clone + std::fmt::Debug + 'a,
{
    print!("\"{:?}\"", &mut root.value());
    for child in root.children.iter_mut() {
        render(child);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rose_trees_hold_lazy_values() {
        let n = 42;
        let mut tree = Tree::singleton(n);
        tree.value();
        tree.value();
        assert_eq!(*tree.value(), n);
    }

    #[test]
    fn trees_get_rendered() {
        let mut tree = Tree::singleton(3);
        tree.children = vec![Tree::singleton(5)];
        render(&mut tree);
    }
}
