use lazy::Lazy;
use std::rc::Rc;

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
pub fn unfold<'a, A, B, F, G>(f: Rc<F>, g: Rc<G>) -> impl Fn(B) -> Tree<'a, A>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(B) -> A,
    G: Fn(B) -> &'a [B],
{
    // This is a bit horrific.
    // We should probably change this unfold into something
    // iterative (non-recursive) as to avoid this nightmare.
    // It may also be worth exploring the use of FnBox, instead.
    move |x: B| {
        let y = f(x.clone());
        Tree {
            thunk: Lazy::new(move || y.clone()),
            children: unfold_forest(f.clone(), g.clone(), x),
        }
    }
}

/// Build a list of trees from an unfolding function and a seed value.
pub fn unfold_forest<'a, A, B, F, G>(f: Rc<F>, g: Rc<G>, x: B) -> Vec<Tree<'a, A>>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(B) -> A,
    G: Fn(B) -> &'a [B],
{
    g(x).iter()
        .map(move |v| unfold(f.clone(), g.clone())(v.clone()))
        .collect()
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
