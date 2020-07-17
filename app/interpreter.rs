use crate::aplang::*;
use crate::aplang::ap;

use std::collections::HashMap;

type Env = HashMap<Var, ApTree>;

pub fn reduce(tree: &ApTree, env : &Env) -> ApTree {
    use Token::*;
    use ApTree::*;
    use ApArity::*;
    let matched_rule =
        match get_arity(&tree) {
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
            Unary(Inc, T(Int(n))) => Some(int (n+1)),
            Unary(Dec, T(Int(n))) => Some(int (n-1)),
            Unary(Modulate, T(Int(n))) => { panic!("Unimplemented Modulate") },
            Unary(Demodulate, T(Int(n))) => { panic!("Unimplemented Demodulate") },
            Unary(Neg, T(Int(n))) => Some(int (-n)),
            Unary(Pwr2, T(Int(n))) => { panic!("Unimplemented Pwr2") },

            // BINARY INTEGER OPERATORS
            Binary(Add, T(Int(a)), T(Int(b))) => Some(int(a+b)),
            Binary(Multiply, T(Int(a)), T(Int(b))) => Some(int(a*b)),
            Binary(Div, T(Int(a)), T(Int(b))) => Some(int(a/b)), //TODO: Correct?

            // COMPARISON ON INTEGERS
            // TODO: Equality on variables? On deep expressions?
            Binary(Eq, T(Int(a)), T(Int(b))) => {
                if a == b {
                    Some(T(True))
                } else {
                    Some(T(False))
                }
            },
            Binary(Lt, T(Int(a)), T(Int(b))) => {
                if a < b {
                    Some(T(True))
                } else {
                    Some(T(False))
                }
            },

            // COMBINATORS
            Ternary(S, x, y, z) => {
                let xz = ap(x.clone(), z.clone());
                let yz = ap(y.clone(), z.clone());
                Some(reduce(&ap(xz, yz), &env))
            },
            Ternary(C, x, y, z) => {
                Some(reduce(&ap(ap(x.clone(), y.clone()), z.clone()), &env))
            },
            Ternary(B, x, y, z) => {
                Some(reduce(&ap(x.clone(), ap(y.clone(), z.clone())), &env))
            },
            Unary(I, arg) => Some(reduce(arg, &env)),
            Ternary(If0, T(Int(0)), x, y) => Some(reduce(x, &env)),
            Ternary(If0, T(Int(1)), x, y) => Some(reduce(y, &env)),

            // LISTS
            // Cons: TODO: Any action?
            Unary(Car, T(Nil)) => Some(T(True)),
            Unary(Car, arg) => {
                match get_arity(&reduce(arg, &env)) {
                    Binary(Cons, car, _)  => Some(reduce(car, &env)),
                    _ => panic!("Unimplemented: Car as free ap")
                }
            },
            Unary(Cdr, T(Nil)) => Some(T(True)),
            Unary(Cdr, arg) => {
                match get_arity(&reduce(arg, &env)) {
                    Binary(Cons, _, cdr)  => Some(reduce(cdr, &env)),
                    _ => panic!("Unimplemented: Cdr as free ap")
                }
            },
            Binary(Vec, cdr, car) => Some(reduce(&cons(cdr.clone(), car.clone()), &env)),
            // Nil: TODO: Any action?
            Unary(IsNil, T(Nil)) => Some(T(True)),
            Unary(IsNil, arg) => {
                match get_arity(&reduce(arg, &env)) {
                    Binary(Cons, _, _) => Some(T(False)),
                    _ => panic!("Undefined IsNil")
                }
            }

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

            _ => None
        };
    match matched_rule {
        Some(new_tree) => new_tree,
        None => {
            match &tree {
                Ap(body) => {
                    let (oper, arg) = body.as_ref();
                    let roper = reduce(oper, &env);
                    let rarg = reduce(arg, &env);
                    if &roper != oper || &rarg != arg {
                        reduce(&ap(roper, rarg), &env)
                    } else {
                        ap(roper, rarg)
                    }
                },
                T(V(_)) => panic!("This should be impossible"),
                T(Int(_)) => panic!("This should be impossible"),
                T(token) => T(*token),

            }
        }
    }
}


#[derive(Debug)]
pub enum ApPartial {
    PendingBoth,
    PendingRight(ApTree),
    Tree(ApTree),
}

type PartialStack = Vec<ApPartial>;

fn words_to_tree(words: &Vec<Word>) -> ApTree {
    use Word::*;
    use ApPartial::*;
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
                        },
                        Some(PendingBoth) => {
                            stack.push(PendingRight(top));
                            break;
                        },
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
        _ => panic!("Interpreted expression did not collapse to a tree: {:#?}", stack[0]),
    }
}

pub fn interpret_program(program : &Program) -> (ApTree, Env) {
    let mut env = Env::default();
    let mut last_var = Var(-100); // Magic?
    for (var, words) in program {
        let expr = words_to_tree(words);
        env.insert(*var, expr);
        last_var = *var;
    }
    (reduce(env.get(&last_var).unwrap(), &env), env)
}



#[cfg(test)]
mod test {
    use super::*;
    use ApTree::*;
    use Token::*;
    use Word::*;

    #[test]
    fn test_words_to_tree() {
        let env = Env::default();
        assert_eq!(words_to_tree(&vec![WT(Int(1))]),
                   int(1));
        assert_eq!(reduce(&words_to_tree(&vec![WAp, WT(Add), WT(Int(1))]), &env),
                   ap(T(Add), int(1)));
        assert_eq!(reduce(&words_to_tree(&vec![WAp, WT(Inc), WT(Int(1))]), &env),
                   T(Int(2)));
    }
      
}
