use lazy::Lazy;
use std::borrow::Borrow;
use std::fmt;
use std::fmt::{Debug, Display, Write};
use std::rc::Rc;

pub struct Tree<A> {
    // TODO: Drop Lazy if this works.
    thunk: A,
    pub children: Box<dyn Iterator<Item = Tree<A>>>,
}

impl<A> Tree<A> {
    pub fn new(value: A, children: Box<dyn Iterator<Item = Tree<A>>>) -> Self {
        let thunk = Lazy::new(value);
        Tree { thunk, children }
    }

    pub fn singleton(value: A) -> Tree<A> {
        Tree {
            thunk: Lazy::new(value),
            children: Box::new(vec![].into_iter()),
        }
    }

    pub fn value(&self) -> A {
        self.thunk
    }

    pub fn expand<F>(f: Rc<F>, t: Tree<A>) -> Tree<A>
    where
        F: Fn(A) -> Box<dyn Iterator<Item = A>>,
    {
        let mut children = Box::new(t.children.map(|t| Self::expand(f, t)));
        let zs = unfold_forest(Rc::new(move |x| x), f, t.value());
        children.chain(zs);
        Tree::new(t.value(), children)
    }
}

pub fn bind<A, B, F>(t: Tree<A>, k: Rc<F>) -> Tree<B>
where
    F: Fn(A) -> Tree<B>,
{
    let x = t.value();
    let xs0 = t.children;
    let mut t1 = k(x);
    let xs = Box::new(xs0.map(|m| bind(m, k)));
    xs.chain(t1.children);
    Tree {
        thunk: t1.thunk,
        children: xs,
    }
}

pub fn join<A>(tss: Tree<Tree<A>>) -> Tree<A> {
    bind(tss, Rc::new(move |x| x))
}

pub fn duplicate<A>(t: Tree<A>) -> Tree<Tree<A>> {
    let xs = Box::new(t.children.map(|x| duplicate(x)));
    Tree::new(t, xs)
}

pub fn fold<A, X, B, F, G>(f: &F, g: &G, t: Tree<A>) -> B
where
    F: Fn(A, X) -> B,
    G: Fn(Box<dyn Iterator<Item = B>>) -> X,
{
    let x = t.value();
    let xs = t.children;
    f(x, fold_forest(f, g, xs))
}

pub fn fold_forest<A, X, B, F, G>(f: &F, g: &G, xs: Box<dyn Iterator<Item = Tree<A>>>) -> X
where
    F: Fn(A, X) -> B,
    G: Fn(Box<dyn Iterator<Item = B>>) -> X,
{
    g(Box::new(xs.map(|x| fold(f, g, x))))
}

impl<A> PartialEq for Tree<A>
where
    A: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
            && self
                .children
                .zip(other.children.collect::<Vec<Tree<A>>>())
                .all(|(x, y)| x.value() == y.value())
    }
}

/// Build a tree from an unfolding function and a seed value.
pub fn unfold<A, B, F, G>(f: Rc<F>, g: Rc<G>, x: B) -> Tree<A>
where
    F: Fn(B) -> A,
    G: Fn(B) -> Box<dyn Iterator<Item = B>>,
{
    let y = f(x);
    Tree {
        thunk: Lazy::new(y),
        children: unfold_forest(f, g, x),
    }
}

/// Build a list of trees from an unfolding function and a seed value.
pub fn unfold_forest<A, B, F, G>(f: Rc<F>, g: Rc<G>, x: B) -> Box<dyn Iterator<Item = Tree<A>>>
where
    F: Fn(B) -> A,
    G: Fn(B) -> Box<dyn Iterator<Item = B>>,
{
    Box::new(g(x).map(move |v| unfold(f, g, v)))
}

// TODO: Not sure if this a poor pattern.
impl<A> AsRef<Tree<A>> for Tree<A> {
    fn as_ref(&self) -> &Tree<A> {
        self.borrow()
    }
}

// TODO: iiuc this is just `value`.
// TODO: https://github.com/hedgehogqa/fsharp-hedgehog/blob/master/src/Hedgehog/Tree.fs#L12-L13
pub fn outcome<A, T>(t: T) -> A
where
    T: AsRef<Tree<A>>,
{
    t.as_ref().value()
}

pub fn shrinks<A>(t: Tree<A>) -> Box<dyn Iterator<Item = Tree<A>>> {
    t.children
}

// TODO: https://github.com/hedgehogqa/fsharp-hedgehog/blob/master/src/Hedgehog/Tree.fs#L84-L87
pub fn filter<A, F>(f: Rc<F>, t: Tree<A>) -> Tree<A>
where
    F: Fn(A) -> bool,
{
    Tree::new(t.value(), filter_forest(f, t.children))
}

pub fn filter_forest<A, F>(
    f: Rc<F>,
    xs: Box<dyn Iterator<Item = Tree<A>>>,
) -> Box<dyn Iterator<Item = Tree<A>>>
where
    F: Fn(A) -> bool,
{
    Box::new(xs.filter(|x| f(outcome(x))).map(|x| filter(f, x)))
}

pub fn map<A, B, F>(f: Rc<F>, t: Tree<A>) -> Tree<B>
where
    A: 'static,
    F: Fn(A) -> B + 'static,
{
    let x = f(t.value());
    let xs = Box::new(t.children.map(move |c| map(f, c)));
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

fn render_forest_lines<A>(limit: i16, forest: &[Tree<A>]) -> Vec<String>
where
    A: Debug,
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

fn render_tree_lines<A>(limit: i16, x: &Tree<A>) -> Vec<String>
where
    A: Debug,
{
    let mut children: Vec<String> =
        render_forest_lines(limit, &x.children.collect::<Vec<Tree<A>>>());
    let node = format!(" {:?}", x.value());

    children.insert(0, node);
    children
}

impl<A> Display for Tree<A>
where
    A: Copy + Debug,
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
