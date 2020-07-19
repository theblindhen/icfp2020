use std::fmt;

use crate::aplang::*;
use crate::nom_helpers::*;
use std::iter;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueTree {
    VNil,
    VInt(num_bigint::BigInt),
    VCons(Box<(ValueTree, ValueTree)>),
}

impl fmt::Display for ValueTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ValueTree::*;

        let is_cons_list = {
            let mut node = self;
            while let VCons(pair) = node {
                node = &pair.1;
            }
            node == &VNil
        };

        if is_cons_list {
            write!(f, "[")?;
            let mut node = self;
            while let VCons(pair) = node {
                write!(f, "{}", pair.0)?;
                node = &pair.1;
                if node != &VNil {
                    write!(f, ", ")?;
                }
            }
            write!(f, "]")?;

        } else {
            match self {
                VCons(pair) => write!(f, "({}, {})", pair.0, pair.1)?,
                VInt(i) => write!(f, "{}", i)?,
                VNil => panic!("Impossible: nil is a cons list"),
            }
        }
        Ok(())
    }
}

impl From<&ValueTree> for ApTree {
    fn from(t: &ValueTree) -> Self {
        use ApTree::{T, Ap};
        match t {
            ValueTree::VNil => T(Token::Nil),
            ValueTree::VCons(pair) => Ap(Box::new((Ap(Box::new((T(Token::Cons), (&pair.as_ref().0).into()))), (&pair.as_ref().1).into()))),
            ValueTree::VInt(i) => T(Token::Int(i.clone())),
        }
    }
}

impl From<ValueTree> for ApTree {
    fn from(t: ValueTree) -> Self { (&t).into() }
}

pub fn parse_value_tree(s: &str) -> Option<ValueTree> {
    match value_tree(s) {
        Ok((_, tree)) => Some(tree),
        _ => None
    }
}

fn value_tree(s: &str) -> IResult<&str, ValueTree> {
    let parse_pair =
        n::delimited(
            n::tag("("),
            n::separated_pair(value_tree, n::tag(", "), value_tree),
            n::tag(")")
        );

    let parse_int =
        n::map_res(decint, |s: &str| s.parse());

    let parse_cons_list =
        n::map(
            n::delimited(
                n::tag("["),
                n::separated_list(n::tag(", "), value_tree),
                n::tag("]")
            ),
            |trees| {
                let mut result = ValueTree::VNil;
                for tree in trees.into_iter().rev() {
                    result = ValueTree::VCons(Box::new((tree, result)));
                }
                result
            }
        );

    n::alt((
        n::map(parse_int, ValueTree::VInt),
        n::map(parse_pair, |pair| ValueTree::VCons(Box::new(pair))),
        parse_cons_list,
    ))(s)
}

pub struct ConsList<'a>(pub &'a ValueTree);

pub struct ConsIterator<'a>(&'a ValueTree);

impl<'a> iter::Iterator for ConsIterator<'a> {
    type Item = &'a ValueTree;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            ValueTree::VNil => None,
            ValueTree::VCons(pair) => {
                let (head, tail) = pair.as_ref();
                self.0 = tail;
                Some(head)
            }
            _ => panic!("Not a list: {:?}", self.0),
        }
    }
}

impl<'a> iter::IntoIterator for &ConsList<'a> {
    type Item = &'a ValueTree;
    type IntoIter = ConsIterator<'a>;
    fn into_iter(self) -> Self::IntoIter {
        ConsIterator(self.0)
    }
}

pub fn to_native_list(tree: &ValueTree) -> Vec<&ValueTree> {
    let mut vec = vec!();

    for elm in &ConsList(tree) {
        vec.push(elm);
    }

    vec
}