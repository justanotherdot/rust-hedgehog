use lazy::Lazy;
use std::borrow::Borrow;
use std::fmt;
use std::fmt::{Debug, Display, Write};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Tree<'a, A, C>
where
    A: Clone,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    thunk: Lazy<'a, A>,
    pub children: C,
}

impl<'a, A, C> Tree<'a, A, C>
where
    A: 'a + Clone,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    pub fn new(value: A, children: Vec<Tree<'a, A, C>>) -> Self {
        let thunk = Lazy::new(value);
        Tree { thunk, children }
    }

    pub fn singleton(value: A) -> Tree<'a, A, C> {
        Tree {
            thunk: Lazy::new(value),
            children: vec![],
        }
    }

    pub fn value(&self) -> A {
        self.thunk.value()
    }

    pub fn expand<F>(f: Rc<F>, t: Tree<'a, A, C>) -> Tree<'a, A, C>
    where
        F: Fn(A) -> Vec<A>,
    {
        let mut children: Vec<Tree<'a, A, C>> = t
            .children
            .iter()
            .map(|t| Self::expand(f.clone(), t.clone()))
            .collect();
        let mut zs = unfold_forest(Rc::new(move |x| x), f.clone(), t.value());
        children.append(&mut zs);
        Tree::new(t.value(), children)
    }
}

pub fn bind<'a, A, B, F, C>(t: Tree<'a, A, C>, k: Rc<F>) -> Tree<'a, B, C>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(A) -> Tree<'a, B, C> + 'a,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    let x = t.value();
    let xs0 = t.children;
    let mut t1 = k(x.clone());
    let mut xs: Vec<Tree<'a, B, C>> = xs0.iter().map(|m| bind(m.clone(), k.clone())).collect();
    xs.append(&mut t1.children);
    Tree {
        thunk: t1.thunk,
        children: xs,
    }
}

pub fn join<'a, A, C>(tss: Tree<'a, Tree<'a, A, C>, C>) -> Tree<'a, A, C>
where
    A: Clone + 'a,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    bind(tss, Rc::new(move |x| x))
}

pub fn duplicate<'a, A, C>(t: Tree<'a, A, C>) -> Tree<'a, Tree<'a, A, C>, C>
where
    A: Clone + 'a,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    let xs = t
        .clone()
        .children
        .into_iter()
        .map(|x| duplicate(x))
        .collect();
    Tree::new(t, xs)
}

pub fn fold<'a, A, X, B, F, G, C>(f: &F, g: &G, t: Tree<A, C>) -> B
where
    A: Clone,
    B: Clone,
    X: Clone,
    // TODO get rid of these static lifetimes
    F: Fn(A, X) -> B + 'static,
    G: Fn(Vec<B>) -> X + 'static,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    let x = t.value();
    let xs = t.children;
    f(x, fold_forest(f, g, xs))
}

pub fn fold_forest<'a, A, X, B, F, G, C>(f: &F, g: &G, xs: Vec<Tree<A, C>>) -> X
where
    A: Clone,
    B: Clone,
    X: Clone,
    // TODO get rid of these static lifetimes
    F: Fn(A, X) -> B + 'static,
    G: Fn(Vec<B>) -> X + 'static,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    g(xs.into_iter().map(|x| fold(f, g, x)).collect())
}

impl<'a, A, C> PartialEq for Tree<'a, A, C>
where
    A: 'a + Clone + PartialEq,
    C: Iterator<Item = Tree<'a, A, C>>,
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
pub fn unfold<'a, A, B, F, G, C>(f: Rc<F>, g: Rc<G>, x: B) -> Tree<'a, A, C>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(B) -> A,
    G: Fn(B) -> Vec<B>,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    let y = f(x.clone());
    Tree {
        thunk: Lazy::new(y),
        children: unfold_forest(f.clone(), g.clone(), x),
    }
}

/// Build a list of trees from an unfolding function and a seed value.
pub fn unfold_forest<'a, A, B, F, G, C>(f: Rc<F>, g: Rc<G>, x: B) -> Vec<Tree<'a, A, C>>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(B) -> A,
    G: Fn(B) -> Vec<B>,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    g(x).iter()
        .map(move |v| unfold(f.clone(), g.clone(), v.clone()))
        .collect()
}

// TODO: Not sure if this a poor pattern.
impl<'a, A, C> AsRef<Tree<'a, A, C>> for Tree<'a, A, C>
where
    A: Clone + 'a,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    fn as_ref(&self) -> &Tree<'a, A, C> {
        self.borrow()
    }
}

// TODO: iiuc this is just `value`.
// TODO: https://github.com/hedgehogqa/fsharp-hedgehog/blob/master/src/Hedgehog/Tree.fs#L12-L13
pub fn outcome<'a, A, T, C>(t: T) -> A
where
    A: Clone + 'a,
    T: AsRef<Tree<'a, A, C>>,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    t.as_ref().value()
}

pub fn shrinks<'a, A, C>(t: Tree<A, C>) -> Vec<Tree<A, C>>
where
    A: Clone,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    t.children
}

// TODO: https://github.com/hedgehogqa/fsharp-hedgehog/blob/master/src/Hedgehog/Tree.fs#L84-L87
pub fn filter<'a, A, F, C>(f: Rc<F>, t: Tree<'a, A, C>) -> Tree<'a, A, C>
where
    A: Clone + 'a,
    F: Fn(A) -> bool + 'a,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    Tree::new(t.value(), filter_forest(f.clone(), t.children))
}

pub fn filter_forest<'a, A, F, C>(f: Rc<F>, xs: Vec<Tree<'a, A, C>>) -> Vec<Tree<'a, A, C>>
where
    A: Clone + 'a,
    F: Fn(A) -> bool + 'a,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    xs.into_iter()
        .filter(|x| f(outcome(x.clone())))
        .map(|x| filter(f.clone(), x))
        .collect()
}

pub fn map<'a, A, B, F, C>(f: Rc<F>, t: Tree<'a, A, C>) -> Tree<'a, B, C>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(A) -> B,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    let x = f(t.value());
    let xs = t.children.into_iter().map(|c| map(f.clone(), c)).collect();
    Tree::new(x, xs)
}

// should be: shift hd other = zipWith (++) (hd : repeat other)
fn shift(head: &str, other: &str, lines: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    let mut first = true;
    for line in lines {
        if first {
            first = false;
            out.push(format!("{}{}", head, line));
        } else {
            out.push(format!("{}{}", other, line));
        }
    }
    out
}

fn render_forest_lines<'a, A, C>(limit: i16, forest: &[Tree<'a, A, C>]) -> Vec<String>
where
    A: Debug + Clone,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    if limit <= 0 {
        return vec!["...".to_owned()];
    }

    match forest {
        [] => vec![],
        [x] => {
            let s = render_tree_lines(limit - 1, x.as_ref());
            shift(" └╼", "   ", s)
        }
        xs0 => {
            let (x, xs) = xs0.split_at(1);
            let s0 = render_tree_lines(limit - 1, x.first().unwrap().as_ref());
            let ss = render_forest_lines(limit, xs);

            let mut s = shift(" ├╼", " │ ", s0);

            s.extend(ss.into_iter());
            s
        }
    }
}

fn render_tree_lines<'a, A, C>(limit: i16, x: &Tree<'a, A, C>) -> Vec<String>
where
    A: Debug + Clone,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    let mut children: Vec<String> = render_forest_lines(limit, &x.children);
    let node = format!(" {:?}", x.value());

    children.insert(0, node);
    children
}

impl<'a, A, C> Display for Tree<'a, A, C>
where
    A: Copy + Debug,
    C: Iterator<Item = Tree<'a, A, C>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error>
    where
        A: Debug,
    {
        for line in render_tree_lines(100, self) {
            // surely a better way for direct Strings
            f.write_str(&line)?;
            f.write_char('\n')?;
        }

        Ok(())
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
