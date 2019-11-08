use lazy::Lazy;
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
    pub children: Vec<Lazy<'a, Tree<'a, A>>>,
}

impl<'a, A> Tree<'a, A>
where
    A: 'a + Clone,
{
    pub fn new(value: A, children: Vec<Lazy<'a, Tree<'a, A>>>) -> Self {
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

    pub fn expand<F>(f: Rc<F>, t: &'a Lazy<'a, Tree<'a, A>>) -> Lazy<'a, Tree<'a, A>>
    where
        F: Fn(A) -> Vec<A> + 'a,
    {
        Lazy::from_closure(|| {
            let mut children: Vec<Lazy<'a, Tree<'a, A>>> = t
                .value()
                .children
                .iter()
                .map(|t| Self::expand(f.clone(), t))
                .collect();
            let mut zs = unfold_forest(Rc::new(move |x| x), f.clone(), t.value().value());
            children.append(&mut zs);
            Tree::new(t.value().value(), children)
        })
    }
}

pub fn bind<'a, A, B, F>(t: Lazy<'a, Tree<'a, A>>, k: Rc<F>) -> Lazy<'a, Tree<'a, B>>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(A) -> Lazy<'a, Tree<'a, B>> + 'a,
{
    Lazy::from_closure(|| {
        let x = t.value().value();
        let xs0 = t.value().children;
        let mut t1 = k(x);
        let mut xs: Vec<Lazy<'a, Tree<'a, B>>> =
            xs0.iter().map(|m| bind(m.clone(), k.clone())).collect();
        xs.append(&mut t1.value().children);
        Tree {
            thunk: t1.value().thunk,
            children: xs,
        }
    })
}

pub fn join<'a, A>(tss: Lazy<'a, Tree<'a, Lazy<'a, Tree<'a, A>>>>) -> Lazy<'a, Tree<'a, A>>
where
    A: Clone + 'a,
{
    bind(tss, Rc::new(move |x| x))
}

pub fn duplicate<'a, A>(t: Lazy<'a, Tree<'a, A>>) -> Lazy<'a, Tree<'a, Lazy<'a, Tree<'a, A>>>>
where
    A: Clone + 'a,
{
    Lazy::from_closure(|| {
        let xs = t
            .clone()
            .value()
            .children
            .into_iter()
            .map(|x| duplicate(x))
            .collect();
        Tree::new(t, xs)
    })
}

pub fn fold<'a, A, X, B, F, G>(f: &F, g: &G, t: Lazy<'a, Tree<'a, A>>) -> B
where
    A: Clone + 'a,
    B: Clone + 'a,
    X: Clone + 'a,
    // TODO get rid of these static lifetimes
    F: Fn(A, X) -> B + 'a,
    G: Fn(Vec<B>) -> X + 'a,
{
    let x = t.value().value();
    let xs = t.value().children;
    f(x, fold_forest(f, g, xs))
}

pub fn fold_forest<'a, A, X, B, F, G>(f: &F, g: &G, xs: Vec<Lazy<'a, Tree<'a, A>>>) -> X
where
    A: Clone + 'a,
    B: Clone + 'a,
    X: Clone + 'a,
    // TODO get rid of these static lifetimes
    F: Fn(A, X) -> B + 'a,
    G: Fn(Vec<B>) -> X + 'a,
{
    g(xs.into_iter().map(|x| fold(f, g, x)).collect())
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
pub fn unfold<'a, A, B, F, G>(f: Rc<F>, g: Rc<G>, x: B) -> Lazy<'a, Tree<'a, A>>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(B) -> A + 'a,
    G: Fn(B) -> Vec<B> + 'a,
{
    Lazy::from_closure(|| {
        let y = f(x.clone());
        Tree {
            thunk: Lazy::new(y),
            children: unfold_forest(f.clone(), g.clone(), x),
        }
    })
}

/// Build a list of trees from an unfolding function and a seed value.
pub fn unfold_forest<'a, A, B, F, G>(f: Rc<F>, g: Rc<G>, x: B) -> Vec<Lazy<'a, Tree<'a, A>>>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(B) -> A + 'a,
    G: Fn(B) -> Vec<B> + 'a,
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

/// Synonym for `value`.
pub fn outcome<'a, A, T>(t: T) -> A
where
    A: Clone + 'a,
    T: AsRef<Tree<'a, A>>,
{
    t.as_ref().value()
}

pub fn shrinks<A>(t: Tree<A>) -> Vec<Lazy<Tree<A>>>
where
    A: Clone,
{
    t.children
}

// TODO: https://github.com/hedgehogqa/fsharp-hedgehog/blob/master/src/Hedgehog/Tree.fs#L84-L87
pub fn filter<'a, A, F>(f: Rc<F>, t: Lazy<'a, Tree<'a, A>>) -> Lazy<'a, Tree<'a, A>>
where
    A: Clone + 'a,
    F: Fn(A) -> bool + 'a,
{
    Lazy::from_closure(|| {
        Tree::new(
            t.value().value(),
            filter_forest(f.clone(), t.value().children),
        )
    })
}

pub fn filter_forest<'a, A, F>(
    f: Rc<F>,
    xs: Vec<Lazy<'a, Tree<'a, A>>>,
) -> Vec<Lazy<'a, Tree<'a, A>>>
where
    A: Clone + 'a,
    F: Fn(A) -> bool + 'a,
{
    xs.into_iter()
        .filter(|x| f(outcome(x.value())))
        .map(|x| filter(f.clone(), x))
        .collect()
}

pub fn map<'a, A, B, F>(f: Rc<F>, t: Lazy<'a, Tree<'a, A>>) -> Lazy<'a, Tree<'a, B>>
where
    A: Clone + 'a,
    B: Clone + 'a,
    F: Fn(A) -> B + 'a,
{
    Lazy::from_closure(|| {
        let x = f(t.value().value());
        let xs = t
            .value()
            .children
            .into_iter()
            .map(|c| map(f.clone(), c))
            .collect();
        Tree::new(x, xs)
    })
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

fn render_forest_lines<'a, A: 'a>(limit: i16, forest: &[Tree<'a, A>]) -> Vec<String>
where
    A: Debug + Clone,
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

fn render_tree_lines<'a, A: 'a>(limit: i16, x: &Tree<'a, A>) -> Vec<String>
where
    A: Debug + Clone,
{
    let mut children: Vec<String> = render_forest_lines(
        limit,
        &x.children
            .into_iter()
            .map(|x| x.value())
            .collect::<Vec<Tree<'a, A>>>()
            .as_slice(),
    );
    let node = format!(" {:?}", x.value());

    children.insert(0, node);
    children
}

impl<'a, A: 'a> Display for Tree<'a, A>
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
