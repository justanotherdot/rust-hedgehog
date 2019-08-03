use lazy::Lazy;
use std::borrow::Borrow;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Tree<'a, A>
where
    A: Clone,
{
    thunk: Lazy<'a, A>,
    pub children: Lazy<'a, Vec<Tree<'a, A>>>,
}

impl<'a, A> Tree<'a, A>
where
    A: 'a + Clone,
{
    pub fn new(value: A, children: Vec<Tree<'a, A>>) -> Self {
        let thunk = Lazy::new(value);
        let children = Lazy::new(children);
        Tree { thunk, children }
    }

    pub fn new_lazy(thunk: Lazy<'a, A>, children: Lazy<'a, Vec<Tree<'a, A>>>) -> Self {
        Tree { thunk, children }
    }

    pub fn singleton(value: A) -> Tree<'a, A> {
        Tree {
            thunk: Lazy::new(value),
            children: Lazy::new(vec![]),
        }
    }

    pub fn value(&self) -> A {
        self.thunk.value()
    }

    pub fn expand<F>(f: Rc<F>, t: Tree<'a, A>) -> Tree<'a, A>
    where
        F: Fn(A) -> Vec<A> + 'a,
    {
        let thunk2 = t.thunk.clone();
        let children: Lazy<'a, Vec<Tree<'a, A>>> = Lazy::from_closure(move || {
            let mut children: Vec<Tree<'a, A>> = t.children
                .value()
                .iter()
                .map(|t| Self::expand(f.clone(), t.clone()))
                .collect();
            let mut zs = unfold_forest(Rc::new(move |x| x), f.clone(), t.value());
            children.append(&mut zs);
            children
        });
        Tree::new_lazy(thunk2, children)
    }
}

pub fn bind<'a, A, B, F>(t: Tree<'a, A>, k: Rc<F>) -> Tree<'a, B>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(A) -> Tree<'a, B> + 'a,
{
    let t1 = t.clone().thunk.map(k.clone());
    let t2 = t1.clone();
    let xs: Lazy<'a, Vec<Tree<'a, B>>> = Lazy::from_closure(move || {
        let xs0 = &t.children;
        let mut xs: Vec<Tree<'a, B>> = xs0.value().iter()
            .map(|m| bind(m.clone(), k.clone()))
            .collect();
        xs.append(&mut t1.value().children.value());
        xs
    });
    Tree {
        thunk: t2.value().thunk,
        children: xs,
    }
}

pub fn join<'a, A>(tss: Tree<'a, Tree<'a, A>>) -> Tree<'a, A>
where
    A: Clone + 'a,
{
    bind(tss, Rc::new(move |x| x))
}

pub fn duplicate<'a, A>(t: Tree<'a, A>) -> Tree<'a, Tree<'a, A>>
where
    A: Clone + 'a,
{
    let t1 = t.clone();
    let xs: Lazy<'a, Vec<Tree<'a, Tree<'a, A>>>> = Lazy::from_closure(move || t
        .clone()
        .children
        .value()
        .into_iter()
        .map(|x| duplicate(x))
        .collect());
    Tree::new_lazy(Lazy::new(t1), xs)
}

pub fn fold<'a, A, X, B, F, G>(f: &F, g: &G, t: Tree<A>) -> B
where
    A: Clone + 'static,
    B: Clone + 'static,
    X: Clone + 'static,
    // TODO get rid of these static lifetimes
    F: Fn(A, X) -> B + 'static,
    G: Fn(Vec<B>) -> X + 'static,
{
    let x = t.value();
    let xs = t.children;
    f(x, fold_forest(f, g, xs.value()))
}

pub fn fold_forest<'a, A, X, B, F, G>(f: &F, g: &G, xs: Vec<Tree<A>>) -> X
where
    A: Clone + 'static,
    B: Clone + 'static,
    X: Clone + 'static,
    // TODO get rid of these static lifetimes
    F: Fn(A, X) -> B + 'static,
    G: Fn(Vec<B>) -> X + 'static,
{
    g(xs.into_iter()
      .map(|x| fold(f, g, x))
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
                .value()
                .iter()
                .zip(&other.children.value())
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
        children: Lazy::new(unfold_forest(f.clone(), g.clone(), x)),
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
    t.children.value()
}

// TODO: https://github.com/hedgehogqa/fsharp-hedgehog/blob/master/src/Hedgehog/Tree.fs#L84-L87
pub fn filter<'a, A, F>(f: Rc<F>, t: Tree<'a, A>) -> Tree<'a, A>
where
    A: Clone + 'a,
    F: Fn(A) -> bool + 'a,
{
    Tree::new(t.value(), filter_forest(f.clone(), t.children.value()))
}

pub fn filter_forest<'a, A, F>(f: Rc<F>, xs: Vec<Tree<'a, A>>) -> Vec<Tree<'a, A>>
where
    A: Clone + 'a,
    F: Fn(A) -> bool + 'a,
{
    xs.into_iter()
        .filter(|x| f(outcome(x.clone())))
        .map(|x| filter(f.clone(), x))
        .collect()
}

pub fn map<'a, A, B, F>(f: Rc<F>, t: Tree<'a, A>) -> Tree<'a, B>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(A) -> B + 'a,
{
    Tree {
        thunk: t.thunk.map(f.clone()),
        children: t.children.map(
            Rc::new(move |xs: Vec<Tree<'a, A>>|
                    xs.into_iter()
                        .map(|c| map(f.clone(), c))
                        .collect()
            )
        ),
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
