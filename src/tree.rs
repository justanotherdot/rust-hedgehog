use lazy::Lazy;

pub struct Tree<'a, A> {
    thunk: Lazy<'a, A>,
    #[allow(dead_code)]
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

/// Build a tree from an unfolding function and a seed value.
pub fn unfold<'a, A, B>(
    f: Box<Fn(B) -> A>,
    g: Box<Fn(B) -> Vec<B>>,
) -> Box<Fn(B) -> Tree<'a, A> + 'a>
where
    A: Clone + 'a,
    B: 'a,
{
    Box::new(move |x| Tree {
        thunk: Lazy::new(|| f(x)),
        children: unfold_forest(f, g, x),
    })
}

/// Build a list of trees from an unfolding function and a seed value.
pub fn unfold_forest<'a, A, B>(
    f: Box<Fn(B) -> A>,
    g: Box<Fn(B) -> Vec<B>>,
    x: B,
) -> Vec<Tree<'a, A>>
where
    A: Clone + 'a,
    B: 'a,
{
    g(x).iter().map(|v| unfold(f, g)(*v)).collect()
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
}
