use lazy::Lazy;
use std::borrow::Borrow;
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
    pub fn new(value: A, children: Vec<Tree<'a, A>>) -> Self {
        let thunk = Lazy::new(value);
        Tree { thunk, children }
    }

    pub fn singleton(value: A) -> Tree<'a, A> {
        Tree {
            thunk: Lazy::new(value),
            children: vec![],
        }
    }

    pub fn value(&self) -> A {
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
            let mut zs = unfold_forest(Rc::new(move |x| x), f.clone(), t.value());
            children.append(&mut zs);
            Tree::new(t.value(), children)
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
        let mut t1 = k(x.clone());
        let mut xs: Vec<Tree<'a, B>> = xs0.iter().map(|m| bind(m.clone())(k.clone())).collect();
        xs.append(&mut t1.children);
        Tree {
            thunk: t1.thunk,
            children: xs,
        }
    }
}

pub fn join<'a, A>(tss: Tree<'a, Tree<'a, A>>) -> Tree<'a, A>
where
    A: Clone + 'a,
{
    bind(tss)(Rc::new(move |x| x))
}

pub fn duplicate<'a, A>(t: Tree<'a, A>) -> Tree<'a, Tree<'a, A>>
where
    A: Clone + 'a,
{
    let xs = t
        .clone()
        .children
        .into_iter()
        .map(|x| duplicate(x))
        .collect();
    Tree::new(t, xs)
}

pub fn fold<A, X, B, F, G>(f: Rc<F>, g: Rc<G>, t: Tree<A>) -> B
where
    A: Clone,
    B: Clone,
    X: Clone,
    // TODO get rid of these static lifetimes
    F: Fn(A) -> Rc<dyn Fn(X) -> B> + 'static,
    G: Fn(Vec<B>) -> X + 'static,
{
    let x = t.value();
    let xs = t.children;
    f(x)(fold_forest(f.clone(), g.clone(), xs))
}

pub fn fold_forest<'a, A, X, B, F, G>(f: Rc<F>, g: Rc<G>, xs: Vec<Tree<A>>) -> X
where
    A: Clone,
    B: Clone,
    X: Clone,
    // TODO get rid of these static lifetimes
    F: Fn(A) -> Rc<dyn Fn(X) -> B> + 'static,
    G: Fn(Vec<B>) -> X + 'static,
{
    g(xs.into_iter()
      .map(|x| fold(f.clone(), g.clone(), x))
      .collect())
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
pub fn unfold<'a, A, B, F, G>(f: Rc<F>, g: Rc<G>, x: B) -> Tree<'a, A>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(B) -> A,
    G: Fn(B) -> Vec<B>,
{
    let y = f(x.clone());
    Tree {
        thunk: Lazy::new(y),
        children: unfold_forest(f.clone(), g.clone(), x),
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
        .map(move |v| unfold(f.clone(), g.clone(), v.clone()))
        .collect()
}

// TODO: Not sure if this a poor pattern.
impl<'a, A> AsRef<Tree<'a, A>> for Tree<'a, A>
where
    A: Clone + 'a,
{
    fn as_ref(&self) -> &Tree<'a, A> {
        self.borrow()
    }
}

// TODO: iiuc this is just `value`.
// TODO: https://github.com/hedgehogqa/fsharp-hedgehog/blob/master/src/Hedgehog/Tree.fs#L12-L13
pub fn outcome<'a, A, T>(t: T) -> A
where
    A: Clone + 'a,
    T: AsRef<Tree<'a, A>>,
{
    t.as_ref().value()
}

pub fn shrinks<A>(t: Tree<A>) -> Vec<Tree<A>>
where
    A: Clone,
{
    t.children
}

// TODO: https://github.com/hedgehogqa/fsharp-hedgehog/blob/master/src/Hedgehog/Tree.fs#L84-L87
pub fn filter<'a, A, F>(f: Rc<F>) -> impl Fn(Tree<'a, A>) -> Tree<'a, A>
where
    A: Clone + 'a,
    F: Fn(A) -> bool + 'a,
{
    move |t: Tree<'a, A>| Tree::new(t.value(), filter_forest(f.clone())(t.children))
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
        let x = f(t.value());
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
        assert_eq!(tree.value(), n);
    }
}
