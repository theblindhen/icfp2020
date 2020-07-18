use crate::aplang::*;
use crate::draw;
use crate::lexer::*;

use log::*;
use std::collections::HashMap;
use std::convert::TryInto;
use std::iter;

type Env = HashMap<Var, ApTree>;

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

fn reduce_left_loop(tree: &ApTree, env: &Env) -> WorkTree {
    use Reduction::*;
    let mut red = reduce_tree(tree, &env);
    loop {
        match red {
            Id(wtree) => return wtree,
            Step(wtree) => red = reduce_one(wtree, &env),
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

fn reduce_tree(tree: &ApTree, env: &Env) -> Reduction {
    use ApArity::*;
    use ApTree::*;
    use Reduction::*;
    use WorkTree::*;

    match &tree {
        T(Token::V(v)) => Step(reduce_left_loop(&env.get(&v).unwrap().clone(), env)),
        T(token) => Id(WorkT(*token)),
        Ap(body) => {
            let (oper, arg) = body.as_ref();
            Step(explicit_ap(
                reduce_left_loop(&oper.clone(), &env),
                arg.clone(),
            ))
        }
    }
}

fn reduce_one(wtree: WorkTree, env: &Env) -> Reduction {
    use Reduction::*;
    use Token::*;
    use WorkTree::*;

    match &wtree {
        WorkT(_) => Id(wtree),

        // Unary functions
        Ap1(fun, arg) if is_eager_fun1(*fun) => match (fun, reduce_left_loop(&arg, &env)) {
            (Inc, WorkT(Int(n))) => Step(WorkT(Int(n + 1))),
            (Dec, WorkT(Int(n))) => Step(WorkT(Int(n - 1))),
            (Neg, WorkT(Int(n))) => Step(WorkT(Int(-n))),
            (Pwr2, WorkT(Int(n))) => panic!("Unimplemented pwr2"),
            (I, body) => Step(body),
            (Car, Ap2(Cons, car, _)) => Step(reduce_left_loop(&car, &env)),
            (Cdr, Ap2(Cons, _, cdr)) => Step(reduce_left_loop(&cdr, &env)),
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
                reduce_left_loop(&left, &env),
                reduce_left_loop(&right, &env),
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
        Ap2(True, left, right) => Step(reduce_left_loop(&left, &env)),
        Ap2(False, left, right) => Step(reduce_left_loop(&right, &env)),
        // Lazy binary functions
        Ap2(Cons, _, _) | Ap2(Vec, _, _) => Id(wtree),

        // Higher arity functions are untouched on Ap2
        Ap2(S, _, _)
        | Ap2(C, _, _)
        | Ap2(B, _, _)
        | Ap2(If0, _, _)
            => Id(wtree),

        Ap3(S, x, y, z) => {
            let xz = reduce_left_loop(&ap(x.clone(), z.clone()), &env);
            Step(explicit_ap(xz, ap(y.clone(), z.clone())))
        }
        Ap3(C, x, y, z) => {
            let xz = reduce_left_loop(&ap(x.clone(), z.clone()), &env);
            Step(explicit_ap(xz, y.clone()))
        }
        Ap3(B, x, y, z) => {
            let x = reduce_left_loop(&x, &env);
            Step(explicit_ap(x, ap(y.clone(), z.clone())))
        }
        Ap3(If0, cond, left, right) => {
            match reduce_left_loop(&cond, &env) {
                WorkT(Int(0)) => Step(reduce_left_loop(&left, &env)),
                WorkT(Int(1)) => Step(reduce_left_loop(&right, &env)),
                 _ => panic!("If0 applied illegal conditional")
            }
        }
        e => panic!("Unimplemented: {:#?}", e),
    }
}

pub fn reduce(tree: &ApTree, env: &Env) -> ApTree {
    use ApArity::*;
    use ApTree::*;
    use Token::*;
    let matched_rule = match get_arity(&tree) {
        // VARS
        ZeroAry(V(var)) => Some(reduce(env.get(&var).unwrap(), &env)),
        // Unary(V(var), arg) => reduce(&ap(env.get(&var).unwrap().clone(), arg.clone()), &env),
        // Binary(V(var), arg1, arg2) => reduce(&ap(ap(env.get(&var).unwrap().clone(),
        //                                             arg1.clone()),
        //                                          arg2.clone()),
        //                                      &env),
        // Ternary(V(var), arg1, arg2, arg3) =>
        //     reduce(&ap(ap(ap(env.get(&var).unwrap().clone(),
        //                      arg1.clone()),
        //                   arg2.clone()),
        //                arg3.clone()),
        //            &env),
        // UNARY INTEGER OPERATORS
        Unary(Inc, T(Int(n))) => Some(int(n + 1)),
        Unary(Dec, T(Int(n))) => Some(int(n - 1)),
        Unary(Modulate, T(Int(n))) => panic!("Unimplemented Modulate"),
        Unary(Demodulate, T(Int(n))) => panic!("Unimplemented Demodulate"),
        Unary(Neg, T(Int(n))) => Some(int(-n)),
        Unary(Pwr2, T(Int(n))) => panic!("Unimplemented Pwr2"),

        // BINARY INTEGER OPERATORS
        Binary(Add, T(Int(a)), T(Int(b))) => Some(int(a + b)),
        Binary(Multiply, T(Int(a)), T(Int(b))) => Some(int(a * b)),
        Binary(Div, T(Int(a)), T(Int(b))) => Some(int(a / b)), //TODO: Correct?

        // COMPARISON ON INTEGERS
        // TODO: Equality on variables? On deep expressions?
        Binary(Eq, T(Int(a)), T(Int(b))) => {
            if a == b {
                Some(T(True))
            } else {
                Some(T(False))
            }
        }
        Binary(Lt, T(Int(a)), T(Int(b))) => {
            if a < b {
                Some(T(True))
            } else {
                Some(T(False))
            }
        }

        // COMBINATORS
        Ternary(S, x, y, z) => {
            let xz = ap(x.clone(), z.clone());
            let yz = ap(y.clone(), z.clone());
            Some(reduce(&ap(xz, yz), &env))
        }
        Ternary(C, x, y, z) => Some(reduce(&ap(ap(x.clone(), y.clone()), z.clone()), &env)),
        Ternary(B, x, y, z) => Some(reduce(&ap(x.clone(), ap(y.clone(), z.clone())), &env)),
        Unary(I, arg) => Some(reduce(arg, &env)),
        Ternary(If0, T(Int(0)), x, y) => Some(reduce(x, &env)),
        Ternary(If0, T(Int(1)), x, y) => Some(reduce(y, &env)),

        // LISTS
        // Cons: TODO: Any action?
        Unary(Car, T(Nil)) => Some(T(True)),
        Unary(Car, arg) => match get_arity(&reduce(arg, &env)) {
            Binary(Cons, car, _) => Some(reduce(car, &env)),
            _ => panic!("Unimplemented: Car as free ap"),
        },
        Unary(Cdr, T(Nil)) => Some(T(True)),
        Unary(Cdr, arg) => match get_arity(&reduce(arg, &env)) {
            Binary(Cons, _, cdr) => Some(reduce(cdr, &env)),
            _ => panic!("Unimplemented: Cdr as free ap"),
        },
        Binary(Vec, cdr, car) => Some(reduce(&cons(cdr.clone(), car.clone()), &env)),
        // Nil: TODO: Any action?
        Unary(IsNil, T(Nil)) => Some(T(True)),
        Unary(IsNil, arg) => match get_arity(&reduce(arg, &env)) {
            Binary(Cons, _, _) => Some(T(False)),
            _ => panic!("Undefined IsNil"),
        },

        // DRAWING
        //Draw
        //Checkerboard
        //DrawList

        // PROTOCOL
        //Send

        // TooManyAry(oper, arg) => {
        //     let roper = reduce(oper, &env);
        //     let rarg = reduce(arg, &env);
        //     if &roper != oper || &rarg != arg {
        //         reduce(&ap(roper, rarg), &env)
        //     } else {
        //         ap(roper, rarg)
        //     }
        // }
        _ => None,
    };
    match matched_rule {
        Some(new_tree) => new_tree,
        None => match &tree {
            Ap(body) => {
                let (oper, arg) = body.as_ref();
                let roper = reduce(oper, &env);
                let rarg = reduce(arg, &env);
                if &roper != oper || &rarg != arg {
                    reduce(&ap(roper, rarg), &env)
                } else {
                    ap(roper, rarg)
                }
            }
            T(V(_)) => panic!("This should be impossible"),
            T(Int(_)) => panic!("This should be impossible"),
            T(token) => T(*token),
        },
    }
}



#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueTree {
    VNil,
    VInt(i64),
    VCons(Box<(ValueTree, ValueTree)>),
    VVec(Box<(ValueTree, ValueTree)>),
}

fn work_to_value_tree(tree: WorkTree, env: &Env) -> ValueTree {
    use ValueTree::*;
    use Token::*;
    use WorkTree::*;
    match tree {
        WorkT(Int(val)) => VInt(val),
        WorkT(Nil) => VNil,
        Ap2(Cons, left, right) => VCons(Box::new((
            work_to_value_tree(reduce_left_loop(&left, &env), &env),
            work_to_value_tree(reduce_left_loop(&right, &env), &env),
        ))),
        Ap2(Vec, left, right) => VVec(Box::new((
            work_to_value_tree(reduce_left_loop(&left, &env), &env),
            work_to_value_tree(reduce_left_loop(&right, &env), &env),
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

pub fn interpret_program(program: &Program) -> (ApTree, Env) {
    let mut env = Env::default();
    let mut last_var = Var(-100); // Magic?
    for (var, words) in program {
        let expr = words_to_tree(words);
        env.insert(*var, expr);
        last_var = *var;
    }
    (env.get(&last_var).unwrap().clone(), env)
}

pub fn initial_state() -> ValueTree { ValueTree::VNil }

pub fn interact(protocol: &ApTree, env: &Env, state: &ValueTree, point: draw::Point) -> (ValueTree, draw::Screen) {
    let (new_state, draw_data) = interact0(&protocol, &env, state.clone(), point);
    let screen = draw::image_from_list(PointCollection(&draw_data));
    (new_state, screen)
}

fn interact0(protocol: &ApTree, env: &Env, mut state: ValueTree, point: draw::Point) -> (ValueTree /* newState */, ValueTree /* draw data */) {
    use ApTree::T;
    use Word::WT;
    use ValueTree::*;

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
                let (head, tail) = pair.as_ref();
                self.0 = tail;
                let (x, y) = match head {
                    ValueTree::VCons(pair) => {
                        let (xy, unexplained_nil) = pair.as_ref();
                        assert_eq!(unexplained_nil, &ValueTree::VNil); // TODO: why?
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
                //let (head, tail) = pair.as_ref();
                //let (x, y) = match head {
                //    ValueTree::VCons(pair) | ValueTree::VVec(pair) => pair.as_ref(),
                //    _ => panic!("Not a point"),
                //};
                //self.0 = tail;
                //let (x, y) = match (x, y) {
                //    (ValueTree::VInt(x), ValueTree::VInt(y)) => (*x, *y),
                //    _ => panic!("Not ints: ({:?}, {:?})", x, y),
                //};
                //Some(draw::Point(x.try_into().unwrap(), y.try_into().unwrap()))
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

    // TODO: define modulate for ValueTree instead of ApTree
    let body = crate::encodings::modulate(&data.into());
    trace!("POSTing: {}", body);

    let reply = ureq::post(url)
        .query("apiKey", "91bf0ff907084b7595841e534276a415")
        .send_string(&body)
        .into_string()
        .expect("HTTP POST failed");

    trace!("Got POST reply: {}", reply);
    let env = Env::new();
    // TODO: define demodulate to ValueTree instead of ApTree
    work_to_value_tree(reduce_left_loop(&crate::encodings::demodulate(&reply).0, &env), &env)
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

    // #[test]
    // fn test_words_to_tree() {
    //     let env = Env::default();
    //     assert_eq!(words_to_tree(&vec![WT(Int(1))]),
    //                wint(1));
    //     assert_eq!(reduce(&words_to_tree(&vec![WAp, WT(Add), WT(Int(1))]), &env),
    //                ap(T(Add), int(1)));
    //     assert_eq!(reduce(&words_to_tree(&vec![WAp, WT(Inc), WT(Int(1))]), &env),
    //                T(Int(2)));
    // }

    #[test]
    fn test_reduce_left_loop() {
        let mut env = Env::default();
        env.insert(Var(99), tree_of_str("ap :99 :99")); // diverges
        assert_eq!(reduce_left_loop(&tree_of_str("ap inc 0"), &env), wint(1));
        // assert_eq!(
        //     reduce_left_loop(&tree_of_str("ap add ap inc 0"), &env),
        //     wap1(Add, int(1))
        // );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap ap add 0 1"), &env),
            wint(1)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap ap add 0 ap inc 0"), &env),
            wint(1)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap ap add ap inc 1 ap inc 0"), &env),
            wint(3)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap ap add ap ap add 0 ap inc 0 2"), &env),
            wint(3)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap ap t 1 :99"), &env),
            wint(1)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap ap f :99 1"), &env),
            wint(1)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap ap ap s add inc 1"), &env),
            wint(3)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap ap ap s t :99 1"), &env),
            wint(1)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap car ap ap cons 0 nil"), &env),
            wint(0)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap car ap ap cons 0 :99"), &env),
            wint(0)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap isnil nil"), &env),
            wbool(true)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap isnil ap ap cons :99 :99"), &env),
            wbool(false)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap ap ap c lt 0 1"), &env),
            wbool(false)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap ap ap c t :99 1"), &env),
            wint(1)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap ap ap ap b f :99 :99 1"), &env),
            wint(1)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap ap ap if0 0 1 :99"), &env),
            wint(1)
        );
        assert_eq!(
            reduce_left_loop(&tree_of_str("ap ap ap if0 1 :99 1"), &env),
            wint(1)
        );
    }
}
