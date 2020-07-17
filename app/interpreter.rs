use crate::aplang::*;
use crate::aplang::ap;

use std::collections::HashMap;

type Env = HashMap<Var, ApTree>;

fn reduce(tree: &ApTree, env : &Env) -> ApTree {
    use Token::*;
    use ApTree::*;
    use ApArity::*;
    let mut tree = tree;
    match get_arity(&tree) {
        // TOKENS AND VARS
        ZeroAry(V(var)) => reduce(env.get(&var).unwrap(), &env),

        // UNARY INTEGER OPERATORS
        Unary(Inc, T(Int(n))) => int (n+1),
        Unary(Dec, T(Int(n))) => int (n-1),
        Unary(Modulate, T(Int(n))) => { panic!("Unimplemented Modulate") },
        Unary(Demodulate, T(Int(n))) => { panic!("Unimplemented Demodulate") },
        Unary(Neg, T(Int(n))) => int (-n),
        Unary(Pwr2, T(Int(n))) => { panic!("Unimplemented Pwr2") },

        // BINARY INTEGER OPERATORS
        Binary(Add, T(Int(a)), T(Int(b))) => int(a+b),
        Binary(Multiply, T(Int(a)), T(Int(b))) => int(a*b),
        Binary(Div, T(Int(a)), T(Int(b))) => int(a/b), //TODO: Correct?

        // COMPARISON ON INTEGERS
        // TODO: Equality on variables? On deep expressions?
        Binary(Eq, T(Int(a)), T(Int(b))) => {
            if a == b {
                T(True)
            } else {
                T(False)
            }
        },
        Binary(Lt, T(Int(a)), T(Int(b))) => {
            if a < b {
                T(True)
            } else {
                T(False)
            }
        },

        // COMBINATORS
        Ternary(S, x, y, z) => {
            let xz = ap(x.clone(), z.clone());
            let yz = ap(y.clone(), z.clone());
            reduce(&ap(xz, yz), &env)
        },
        Ternary(C, x, y, z) => {
            reduce(&ap(ap(x.clone(), y.clone()), z.clone()), &env)
        },
        Ternary(B, x, y, z) => {
            reduce(&ap(x.clone(), ap(y.clone(), z.clone())), &env)
        },
        Unary(I, arg) => reduce(arg, &env),
        Ternary(If0, T(Int(0)), x, y) => reduce(x, &env),
        Ternary(If0, T(Int(1)), x, y) => reduce(y, &env),

        // LISTS
        // Cons: TODO: Any action?
        Unary(Car, T(Nil)) => T(True),
        Unary(Car, arg) => {
            match get_arity(&reduce(arg, &env)) {
                Binary(Cons, car, _)  => car.clone(),
                _ => panic!("Unimplemented: Car as free ap")
            }
        },
        Unary(Cdr, T(Nil)) => T(True),
        Unary(Cdr, arg) => {
            match get_arity(&reduce(arg, &env)) {
                Binary(Cons, _, cdr)  => cdr.clone(),
                _ => panic!("Unimplemented: Cdr as free ap")
            }
        },
        Binary(Vec, cdr, car) => reduce(&cons(cdr.clone(), car.clone()), &env),
        // Nil: TODO: Any action?
        Unary(IsNil, T(Nil)) => T(True),
        Unary(IsNil, arg) => {
            match get_arity(&reduce(arg, &env)) {
                Binary(Cons, _, _) => T(False),
                _ => panic!("Undefined IsNil")
            }
        }

        // DRAWING
        //Draw
        //Checkerboard
        //DrawList

        // PROTOCOL
        //Send

        

        _ => tree.clone()
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

pub fn interpret_program(program : &Program) -> ApTree {
    let mut env = Env::default();
    let mut last_var = Var(-100); // Magic?
    for (var, words) in program {
        let expr = words_to_tree(words);
        env.insert(*var, expr);
        last_var = *var;
    }
    reduce(env.get(&last_var).unwrap(), &env)
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
