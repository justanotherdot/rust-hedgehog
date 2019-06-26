use lazy::Lazy;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Tree<'a, A>
where
    A: Clone,
{
    thunk: Lazy<'a, A>,
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

pub fn bind<'a, A, B, F>(t: Tree<'a, A>) -> impl Fn(Rc<F>) -> Tree<'a, B>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(A) -> Tree<'a, B> + 'a,
{
    let x = t.value();
    let xs0 = t.children;
    move |k: Rc<F>| {
        let mut t1 = k(x.clone().unwrap());
        let mut xs: Vec<Tree<'a, B>> = xs0.iter().map(|m| bind(m.clone())(k.clone())).collect();
        xs.append(&mut t1.children);
        Tree {
            thunk: t1.thunk,
            children: xs,
        }
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

// TODO: iiuc this is just `value`.
// TODO: https://github.com/hedgehogqa/fsharp-hedgehog/blob/master/src/Hedgehog/Tree.fs#L12-L13
pub fn outcome<'a, A>(t: Tree<'a, A>) -> A
where
    A: Clone + 'a,
{
    t.value().unwrap()
}

// TODO: https://github.com/hedgehogqa/fsharp-hedgehog/blob/master/src/Hedgehog/Tree.fs#L84-L87
pub fn filter<'a, A, F>(f: Rc<F>) -> impl Fn(Tree<'a, A>) -> Tree<'a, A>
where
    A: Clone + 'a,
    F: Fn(A) -> bool + 'a,
{
    move |t: Tree<'a, A>| Tree::new(t.value().unwrap(), filter_forest(f.clone())(t.children))
}

pub fn filter_forest<'a, A, F>(f: Rc<F>) -> impl Fn(Vec<Tree<'a, A>>) -> Vec<Tree<'a, A>>
where
    A: Clone + 'a,
    F: Fn(A) -> bool + 'a,
{
    move |xs: Vec<Tree<'a, A>>| {
        xs.into_iter()
            .filter(|x| f(outcome(x.clone())))
            .map(|x| filter(f.clone())(x))
            .collect()
    }
}

pub fn map<'a, A, B, F>(f: Rc<F>) -> impl Fn(Tree<'a, A>) -> Tree<'a, B>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(A) -> B,
{
    move |t| {
        let x = f(t.value().unwrap());
        let xs = t.children.into_iter().map(|c| map(f.clone())(c)).collect();
        Tree::new(x, xs)
    }
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
