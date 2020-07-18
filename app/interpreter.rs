use crate::aplang::*;
use crate::draw;
use crate::lexer::*;

use log::*;
use std::collections::HashMap;
use std::convert::TryInto;
use std::iter;

enum VarTree {
    Open(ApTree),
    Reduced(WorkTree),
}

#[derive(Default)]
pub struct Env {
    m: HashMap<Var, VarTree>,
    id: i32,
}

impl Env {
    fn insert(&mut self, var : Var, tree : ApTree) {
        self.m.insert(var,  VarTree::Open(tree));
    }

    fn get_and_reduce(&mut self, v: Var) -> WorkTree {
        use VarTree::*;
        match self.m.get(&v) {
            Some(Open(aptree)) => {
                let wtree = reduce_left_loop(&aptree.clone(), self);
                self.m.insert(v, Reduced(wtree.clone()));
                wtree
            },
            Some(Reduced(wtree)) => wtree.clone(),
            None => panic!("Unknown variable")
        }
    }

    fn fresh_var(&mut self) -> Var {
        let i = self.id + 1;
        self.id = i;
        Var(-i)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkTree {
    WorkT(Token),
    Ap1(Token, ApTree),
    Ap2(Token, ApTree, ApTree),
    Ap3(Token, ApTree, ApTree, ApTree),
    // List(Vec<ApTree>)
}

enum Reduction {
    Id(WorkTree),
    Step(WorkTree),
}

fn reduce_left_loop(tree: &ApTree, env: &mut Env) -> WorkTree {
    use Reduction::*;
    let mut red = reduce_tree(tree, env);
    loop {
        match red {
            Id(wtree) => return wtree,
            Step(wtree) => red = reduce_one(wtree, env),
        }
    }
}

fn explicit_ap(fun: WorkTree, arg: ApTree) -> WorkTree {
    use WorkTree::*;
    match fun {
        WorkT(token) => Ap1(token, arg),
        Ap1(token, arg1) => Ap2(token, arg1, arg),
        Ap2(token, arg1, arg2) => Ap3(token, arg1, arg2, arg),
        Ap3(_, _, _, _) => panic!("Met unapplicable Ap3"),
    }
}

fn reduce_tree(tree: &ApTree, env: &mut Env) -> Reduction {
    use ApArity::*;
    use ApTree::*;
    use Reduction::*;
    use WorkTree::*;

    match &tree {
        T(Token::V(v)) => Step(env.get_and_reduce(*v)),
        T(token) => Id(WorkT(*token)),
        Ap(body) => {
            let (oper, arg) = body.as_ref();
            Step(explicit_ap(
                reduce_left_loop(&oper.clone(), env),
                arg.clone(),
            ))
        }
    }
}

fn reduce_one(wtree: WorkTree, env: &mut Env) -> Reduction {
    use Reduction::*;
    use Token::*;
    use WorkTree::*;

    match &wtree {
        WorkT(_) => Id(wtree),

        // Unary functions
        Ap1(fun, arg) if is_eager_fun1(*fun) => match (fun, reduce_left_loop(&arg, env)) {
            (Inc, WorkT(Int(n))) => Step(WorkT(Int(n + 1))),
            (Dec, WorkT(Int(n))) => Step(WorkT(Int(n - 1))),
            (Neg, WorkT(Int(n))) => Step(WorkT(Int(-n))),
            (Pwr2, WorkT(Int(n))) => panic!("Unimplemented pwr2"),
            (I, body) => Step(body),
            (Car, Ap2(Cons, car, _)) => Step(reduce_left_loop(&car, env)),
            (Cdr, Ap2(Cons, _, cdr)) => Step(reduce_left_loop(&cdr, env)),
            (IsNil, WorkT(Nil)) => Step(WorkT(True)),
            (IsNil, Ap2(Cons, _, _)) => Step(WorkT(False)),
            _ => panic!("Eager arg should evaluate to token"),
        },

        // Higher arity functions are untouched on Ap1
        Ap1(True, _)
        | Ap1(False, _)
        | Ap1(Add, _)
        | Ap1(Multiply, _)
        | Ap1(Div, _)
        | Ap1(Eq, _)
        | Ap1(Lt, _)
        | Ap1(S, _)
        | Ap1(C, _)
        | Ap1(B, _)
        | Ap1(If0, _)
        | Ap1(Cons, _)
        | Ap1(Vec, _) => Id(wtree),

        // Binary functions
        Ap2(fun, left, right) if is_eager_fun2(*fun) => {
            match (
                fun,
                reduce_left_loop(&left, env),
                reduce_left_loop(&right, env),
            ) {
                (Add, WorkT(Int(x)), WorkT(Int(y))) => Step(WorkT(Int(x + y))),
                (Multiply, WorkT(Int(x)), WorkT(Int(y))) => Step(WorkT(Int(x * y))),
                (Div, WorkT(Int(x)), WorkT(Int(y))) => Step(WorkT(Int(x / y))),
                (Eq, WorkT(Int(x)), WorkT(Int(y))) => {
                    Step(WorkT(if x == y { True } else { False }))
                }
                (Lt, WorkT(Int(x)), WorkT(Int(y))) => Step(WorkT(if x < y { True } else { False })),
                _ => panic!("Binary integer ops with non-int args"),
            }
        }
        Ap2(True, left, right) => Step(reduce_left_loop(&left, env)),
        Ap2(False, left, right) => Step(reduce_left_loop(&right, env)),
        // Lazy binary functions
        Ap2(Cons, _, _) | Ap2(Vec, _, _) => Id(wtree),

        // Higher arity functions are untouched on Ap2
        Ap2(S, _, _)
        | Ap2(C, _, _)
        | Ap2(B, _, _)
        | Ap2(If0, _, _)
            => Id(wtree),

        Ap3(S, x, y, z) => {
            let zvar = env.fresh_var();
            env.insert(zvar, z.clone());
            let xz = reduce_left_loop(&ap(x.clone(), ApTree::T(V(zvar))), env);
            Step(explicit_ap(xz, ap(y.clone(), ApTree::T(V(zvar)))))
        }
        Ap3(C, x, y, z) => {
            let xz = reduce_left_loop(&ap(x.clone(), z.clone()), env);
            Step(explicit_ap(xz, y.clone()))
        }
        Ap3(B, x, y, z) => {
            let x = reduce_left_loop(&x, env);
            Step(explicit_ap(x, ap(y.clone(), z.clone())))
        }
        Ap3(If0, cond, left, right) => {
            match reduce_left_loop(&cond, env) {
                WorkT(Int(0)) => Step(reduce_left_loop(&left, env)),
                WorkT(Int(1)) => Step(reduce_left_loop(&right, env)),
                 _ => panic!("If0 applied illegal conditional")
            }
        }

        Ap3(Cons, x, y, z)
        | Ap3(Vec, x, y, z) => {
            // ap ap ap cons x0 x1 x2   =   ap ap x2 x0 x1
            let zx = reduce_left_loop(&ap(z.clone(), x.clone()), env);
            trace!("Reduce Cons by Church");
            Step(explicit_ap(zx, y.clone()))
        }

        e => panic!("Unimplemented: {:#?}", e),
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueTree {
    VNil,
    VInt(i64),
    VCons(Box<(ValueTree, ValueTree)>),
    VVec(Box<(ValueTree, ValueTree)>),
}

fn work_to_value_tree(tree: WorkTree, env: &mut Env) -> ValueTree {
    use ValueTree::*;
    use Token::*;
    use WorkTree::*;
    match tree {
        WorkT(Int(val)) => VInt(val),
        WorkT(Nil) => VNil,
        Ap2(Cons, left, right) => VCons(Box::new((
            work_to_value_tree(reduce_left_loop(&left, env), env),
            work_to_value_tree(reduce_left_loop(&right, env), env),
        ))),
        Ap2(Vec, left, right) => VVec(Box::new((
            work_to_value_tree(reduce_left_loop(&left, env), env),
            work_to_value_tree(reduce_left_loop(&right, env), env),
        ))),
        _ => panic!("Non-value work tree")
    }
}

impl From<&ValueTree> for ApTree {
    fn from(t: &ValueTree) -> Self {
        use ApTree::{T, Ap};
        match t {
            ValueTree::VNil => T(Token::Nil),
            ValueTree::VCons(pair) => Ap(Box::new((Ap(Box::new((T(Token::Cons), (&pair.as_ref().0).into()))), (&pair.as_ref().1).into()))),
            ValueTree::VInt(i) => T(Token::Int(*i)),
            ValueTree::VVec(pair) => Ap(Box::new((Ap(Box::new((T(Token::Vec), (&pair.as_ref().0).into()))), (&pair.as_ref().1).into()))),
        }
    }
}

impl From<ValueTree> for ApTree {
    fn from(t: ValueTree) -> Self { (&t).into() }
}

#[derive(Debug)]
pub enum ApPartial {
    PendingBoth,
    PendingRight(ApTree),
    Tree(ApTree),
}

type PartialStack = Vec<ApPartial>;

pub fn words_to_tree(words: &Vec<Word>) -> ApTree {
    use ApPartial::*;
    use Word::*;
    let mut stack = PartialStack::default();
    for token in words {
        match token {
            WAp => stack.push(PendingBoth),
            WT(t) => {
                let mut top = ApTree::T(*t);
                loop {
                    match stack.pop() {
                        None => {
                            stack.push(Tree(top));
                            break;
                        }
                        Some(PendingBoth) => {
                            stack.push(PendingRight(top));
                            break;
                        }
                        Some(PendingRight(left)) => {
                            top = ap(left, top);
                        }
                        Some(Tree(_)) => {
                            panic!("Pushed a tree on a tree");
                        }
                    }
                }
            }
        }
    }
    if stack.len() != 1 {
        panic!("Interpreted expression did not collapse");
    }
    match stack.pop() {
        Some(Tree(tree)) => tree,
        None => panic!("Empty expression?"),
        _ => panic!(
            "Interpreted expression did not collapse to a tree: {:#?}",
            stack[0]
        ),
    }
}

pub fn parse_program(program: &Program) -> (Var, Env) {
    let mut env = Env::default();
    let mut last_var = Var(-9999); // Magic?
    for (var, words) in program {
        let expr = words_to_tree(words);
        env.insert(*var, expr);
        last_var = *var;
    }
    (last_var, env)
}

pub fn initial_state() -> ValueTree { ValueTree::VNil }

pub fn interact(prg_var: Var, env: &mut Env, state: &ValueTree, point: draw::Point) -> (ValueTree, draw::Screen) {
    let (new_state, draw_data) = interact0(prg_var, env, state.clone(), point);
    let screen = draw::image_from_list(PointCollection(&draw_data));
    (new_state, screen)
}

fn interact0(prg_var: Var, env: &mut Env, mut state: ValueTree, point: draw::Point) -> (ValueTree /* newState */, ValueTree /* draw data */) {
    use ApTree::T;
    use Word::WT;
    use ValueTree::*;

    let protocol = ApTree::T(Token::V(prg_var));
    let mut vector = VVec(Box::new((VInt(point.0.into()), VInt(point.1.into()))));
    loop {
        let (flag, new_state, data) = {
            let applied_protocol = ap(ap(protocol.clone(), state.into()), vector.into());
            let tuple = work_to_value_tree(reduce_left_loop(&applied_protocol, env), env);
            match tuple {
                VCons(pair) => {
                    let flag = match pair.0 {
                        VInt(flag) => flag,
                        _ => panic!("Flag is not an int")
                    };
                    match pair.1 {
                        VCons(pair) => {
                            let new_state = pair.0;
                            trace!("newState = {:?}", new_state);
                            match pair.1 {
                                VCons(pair) => {
                                    let data = pair.0;
                                    trace!("data = {:?}", data);
                                    assert_eq!(pair.1, ValueTree::VNil);
                                    (flag, new_state.clone(), data.clone())
                                },
                                _ => panic!("interact: no data"),
                            }
                        },
                        _ => panic!("interact: no new_state"),
                    }
                }
                _ => panic!("interact: no flag"),
            }
        };
        if flag == 0 {
            return (new_state, data);
        }
        state = new_state;
        vector = send(&data);
    }
}

struct PointCollection<'a>(&'a ValueTree);

struct PointIterator<'a>(&'a ValueTree);

impl<'a> iter::Iterator for PointIterator<'a> {
    type Item = draw::Point;
    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            ValueTree::VNil => None,
            ValueTree::VCons(pair) => {
                let (head, unexplained_nil) = pair.as_ref();
                assert_eq!(unexplained_nil, &ValueTree::VNil); // TODO: why?
                let (x, y) = match head {
                    ValueTree::VCons(pair) => {
                        let (xy, next_points) = pair.as_ref();
                        self.0 = next_points;
                        match xy {
                            ValueTree::VCons(pair) | ValueTree::VVec(pair) => pair.as_ref(),
                            _ => panic!("Not a point"),
                        }
                    },
                    _ => panic!("Not the dummy list I expected: {:?}", head)
                };
                let (x, y) = match (x, y) {
                    (ValueTree::VInt(x), ValueTree::VInt(y)) => (*x, *y),
                    _ => panic!("Not ints: ({:?}, {:?})", x, y),
                };
                Some(draw::Point(x.try_into().unwrap(), y.try_into().unwrap()))
            }
            _ => panic!("Not a list"),
        }
    }
}

impl<'a> iter::IntoIterator for &PointCollection<'a> {
    type Item = draw::Point;
    type IntoIter = PointIterator<'a>;
    fn into_iter(self) -> Self::IntoIter {
        PointIterator(self.0)
    }
}

pub fn send(data: &ValueTree) -> ValueTree {
    let url = "https://icfpc2020-api.testkontur.ru/aliens/send";
    println!("Sending request to {}...", url);

    let body = crate::encodings::modulate(data);
    trace!("POSTing: {}", body);

    let reply = ureq::post(url)
        .query("apiKey", "91bf0ff907084b7595841e534276a415")
        .send_string(&body)
        .into_string()
        .expect("HTTP POST failed");

    trace!("Got POST reply: {}", reply);

    crate::encodings::demodulate(&reply).0
}

#[cfg(test)]
mod test {
    use super::*;
    use ApTree::*;
    use Token::*;
    use Word::*;
    use WorkTree::*;

    fn tree_of_str(expr: &str) -> ApTree {
        match crate::lexer::aplist(&(String::from(" ") + expr)) {
            Ok(("", words)) => words_to_tree(&words),
            e => panic!("Lex error {:#?}!", e),
        }
    }

    pub fn wap1(fun: Token, arg: ApTree) -> WorkTree {
        WorkTree::Ap1(fun, arg)
    }

    // pub fn wnil() -> ApTree {
    //     return ApTree::T(Token::Nil);
    // }

    // pub fn wcons(head: ApTree, tail: ApTree) -> ApTree {
    //     return ap(ap(ApTree::T(Token::Cons), head), tail);
    // }

    pub fn wbool(v: bool) -> WorkTree {
        WorkT(if v { True } else { False })
    }

    pub fn wint(val: i64) -> WorkTree {
        WorkT(Int(val))
    }

    #[test]
    fn test_reduce_left_loop() {
        let mut env = Env::default();
        env.insert(Var(99), tree_of_str("ap :99 :99")); // diverges
        assert_eq!(reduce_left_loop(&tree_of_str("ap inc 0"), &mut env), wint(1));
        // assert_eq!(
        //     reduce_left_loop(&tree_of_str("ap add ap inc 0"), &mut env),
        //     wap1(Add, int(1))
        // );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap ap add 0 1"), &mut env),
            wint(1)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap ap add 0 ap inc 0"), &mut env),
            wint(1)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap ap add ap inc 1 ap inc 0"), &mut env),
            wint(3)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap ap add ap ap add 0 ap inc 0 2"), &mut env),
            wint(3)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap ap t 1 :99"), &mut env),
            wint(1)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap ap f :99 1"), &mut env),
            wint(1)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap ap ap s add inc 1"), &mut env),
            wint(3)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap ap ap s t :99 1"), &mut env),
            wint(1)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap car ap ap cons 0 nil"), &mut env),
            wint(0)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap car ap ap cons 0 :99"), &mut env),
            wint(0)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap isnil nil"), &mut env),
            wbool(true)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap isnil ap ap cons :99 :99"), &mut env),
            wbool(false)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap ap ap c lt 0 1"), &mut env),
            wbool(false)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap ap ap c t :99 1"), &mut env),
            wint(1)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap ap ap ap b f :99 :99 1"), &mut env),
            wint(1)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap ap ap if0 0 1 :99"), &mut env),
            wint(1)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap ap ap if0 1 :99 1"), &mut env),
            wint(1)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap car ap ap cons 1 ap ap cons 2 3"), &mut env),
            wint(1)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap car ap cdr ap ap cons 1 ap ap cons 2 3"), &mut env),
            wint(2)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap cdr ap cdr ap ap cons 1 ap ap cons 2 3"), &mut env),
            wint(3)
        );
    }
}
