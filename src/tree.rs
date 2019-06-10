use lazy::Lazy;
use std::fmt::{Debug, Error, Formatter};
use std::rc::Rc;

#[derive(Clone)]
pub struct Tree<'a, A> {
    thunk: Lazy<'a, A>,
    #[allow(dead_code)]
    pub children: Vec<Tree<'a, A>>,
}

impl<'a, A> Tree<'a, A>
where
    A: 'a + Clone,
{
    fn new(value: A, children: Vec<Tree<'a, A>>) -> Self {
        let thunk = Lazy::new(value);
        Tree { thunk, children }
    }

    pub fn singleton(value: A) -> Tree<'a, A> {
        Tree {
            thunk: Lazy::new(value),
            children: vec![],
        }
    }

    pub fn value(&self) -> Option<A> {
        self.thunk.value()
    }

    pub fn expand<F>(f: Rc<F>) -> impl Fn(Tree<'a, A>) -> Tree<'a, A>
    where
        F: Fn(A) -> Vec<A>,
    {
        move |t: Tree<'a, A>| {
            let mut children: Vec<Tree<'a, A>> = t
                .children
                .iter()
                .map(|t| Self::expand(f.clone())(t.clone()))
                .collect();
            let mut zs = unfold_forest(Rc::new(move |x| x), f.clone(), t.value().unwrap());
            children.append(&mut zs);
            Tree::new(t.value().unwrap(), children)
        }
    }
}

impl<'a, A> Debug for Tree<'a, A>
where
    A: 'a + Clone + Debug,
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        // TODO: Actually print the children.
        let has_children_str = if self.children.is_empty() {
            format!("<>")
        } else {
            format!("<children>")
        };
        f.write_str(
            format!(
                "Tree {{ {:?}, {} }}",
                self.clone().value(),
                has_children_str
            )
            .as_str(),
        )
    }
}

impl<'a, A> PartialEq for Tree<'a, A>
where
    A: 'a + Clone + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
            && self
                .children
                .iter()
                .zip(&other.children)
                .all(|(x, y)| x.value() == y.value())
    }
}

/// Build a tree from an unfolding function and a seed value.
pub fn unfold<'a, A, B, F, G>(f: Rc<F>, g: Rc<G>) -> impl Fn(B) -> Tree<'a, A>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(B) -> A,
    G: Fn(B) -> Vec<B>,
{
    // This is a bit horrific.
    // We should probably change this unfold into something
    // iterative (non-recursive) as to avoid this nightmare.
    // It may also be worth exploring the use of FnBox, instead.
    move |x: B| {
        let y = f(x.clone());
        Tree {
            thunk: Lazy::new(y),
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
    G: Fn(B) -> Vec<B>,
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
        let tree = Tree::singleton(n);
        tree.value();
        tree.value();
        assert_eq!(tree.value(), Some(n));
    }
}
