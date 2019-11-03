use lazy::{Lazy, LazyVec};
use std::borrow::Borrow;
use std::fmt;
use std::fmt::{Debug, Display, Write};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Tree<'a, A>
where
    A: Clone,
{
    thunk: Lazy<'a, A>,
    //pub children: Vec<Tree<'a, A>>,
    pub children: LazyVec<'a, Tree<'a, A>>,
}

impl<'a, A> Tree<'a, A>
where
    A: 'a + Clone,
{
    pub fn new(value: A, children: LazyVec<'a, Tree<'a, A>>) -> Self {
        let thunk = Lazy::new(value);
        Tree { thunk, children }
    }

    pub fn singleton(value: A) -> Tree<'a, A> {
        Tree {
            thunk: Lazy::new(value),
            children: LazyVec::empty(),
        }
    }

    pub fn value(&self) -> A {
        self.thunk.value()
    }

    pub fn expand<F>(f: Rc<F>, t: Tree<'a, A>) -> Tree<'a, A>
    where
        F: Fn(A) -> LazyVec<'a, A> + 'a,
    {
        let mut children: LazyVec<Tree<'a, A>> = t
            .children
            .map(move |t: Tree<'a, A>| Self::expand(f.clone(), t.clone()));
        let zs = unfold_forest(Rc::new(move |x| x), f.clone(), t.value());
        let children = children.append(zs);
        Tree::new(t.value(), children)
    }
}

pub fn bind<'a, A, B, F>(t: Tree<'a, A>, k: Rc<F>) -> Tree<'a, B>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(A) -> Tree<'a, B> + 'a,
{
    let x = t.value();
    let xs0 = t.children;
    let t1 = k(x.clone());
    let xs: LazyVec<Tree<'a, B>> = xs0.map(&|m: Tree<'a, A>| bind(m.clone(), k.clone()));
    let xs = xs.append(t1.children);
    Tree {
        thunk: t1.thunk,
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
    let xs = t.clone().children.map(&|x| duplicate(x));
    Tree::new(t, xs)
}

pub fn fold<'a, A, X, B, F, G>(f: &'a F, g: &'a G, t: Tree<'a, A>) -> B
where
    A: Clone + 'a,
    B: Clone + 'a,
    X: Clone + 'a,
    F: Fn(A, X) -> B + 'a,
    G: Fn(LazyVec<'a, B>) -> X + 'a,
{
    let x = t.value();
    let xs = t.children;
    f(x, fold_forest(f, g, xs))
}

pub fn fold_forest<'a, A, X, B, F, G>(f: &'a F, g: &'a G, xs: LazyVec<'a, Tree<'a, A>>) -> X
where
    A: Clone + 'a,
    B: Clone + 'a,
    X: Clone + 'a,
    F: Fn(A, X) -> B + 'a,
    G: Fn(LazyVec<'a, B>) -> X + 'a,
{
    g(xs.map(&|x| fold(f, g, x)))
}

impl<'a, A> PartialEq for Tree<'a, A>
where
    A: 'a + Clone + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
            && self
                .children
                .zip(other.children)
                .all(&|(x, y): (Tree<'a, A>, Tree<'a, A>)| x.value() == y.value())
    }
}

/// Build a tree from an unfolding function and a seed value.
pub fn unfold<'a, A, B, F, G>(f: Rc<F>, g: Rc<G>, x: B) -> Tree<'a, A>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(B) -> A + 'a,
    G: Fn(B) -> LazyVec<'a, B> + 'a,
{
    let y = f(x.clone());
    Tree {
        thunk: Lazy::new(y),
        children: unfold_forest(f.clone(), g.clone(), x),
    }
}

/// Build a list of trees from an unfolding function and a seed value.
pub fn unfold_forest<'a, A, B, F, G>(f: Rc<F>, g: Rc<G>, x: B) -> LazyVec<'a, Tree<'a, A>>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(B) -> A + 'a,
    G: Fn(B) -> LazyVec<'a, B> + 'a,
{
    g(x).map(&|v: B| unfold(f.clone(), g.clone(), v.clone()))
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

pub fn shrinks<A>(t: Tree<A>) -> LazyVec<Tree<A>>
where
    A: Clone,
{
    t.children
}

// TODO: https://github.com/hedgehogqa/fsharp-hedgehog/blob/master/src/Hedgehog/Tree.fs#L84-L87
pub fn filter<'a, A, F>(f: Rc<F>, t: Tree<'a, A>) -> Tree<'a, A>
where
    A: Clone + 'a,
    F: Fn(A) -> bool + 'a,
{
    Tree::new(t.value(), filter_forest(f.clone(), t.children))
}

pub fn filter_forest<'a, A, F>(f: Rc<F>, xs: LazyVec<'a, Tree<'a, A>>) -> LazyVec<'a, Tree<'a, A>>
where
    A: Clone + 'a,
    F: Fn(A) -> bool + 'a,
{
    xs.filter(&|x| f(outcome(x.clone())))
        .map(&|x| filter(f.clone(), x))
}

pub fn map<'a, A, B, F>(f: Rc<F>, t: Tree<'a, A>) -> Tree<'a, B>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(A) -> B + 'a,
{
    let x = f(t.value());
    let xs = t.children.map(move |c| map(f.clone(), c));
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

fn render_forest_lines<'a, A>(limit: i16, forest: &[Lazy<'a, Tree<'a, A>>]) -> Vec<String>
where
    A: Debug + Clone + 'a,
{
    if limit <= 0 {
        return vec!["...".to_owned()];
    }

    match forest {
        [] => vec![],
        [x] => {
            let s = render_tree_lines(limit - 1, x.value().as_ref());
            shift(" └╼", "   ", s)
        }
        xs0 => {
            let (x, xs) = xs0.split_at(1);
            let s0 = render_tree_lines(limit - 1, x.first().unwrap().value().as_ref());
            let ss = render_forest_lines(limit, xs);

            let mut s = shift(" ├╼", " │ ", s0);

            s.extend(ss.into_iter());
            s
        }
    }
}

fn render_tree_lines<'a, A>(limit: i16, x: &Tree<'a, A>) -> Vec<String>
where
    A: Debug + Clone + 'a,
{
    let mut children: Vec<String> = render_forest_lines(limit, x.children.to_vec().as_slice());
    let node = format!(" {:?}", x.value());

    children.insert(0, node);
    children
}

impl<'a, A> Display for Tree<'a, A>
where
    A: Copy + Debug + 'a,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
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
