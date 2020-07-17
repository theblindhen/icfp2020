use crate::aplang::*;
use crate::aplang::ap;

use std::collections::HashMap;

type Env = HashMap<Var, ApTree>;

fn reduce_aptree(tree: ApTree, env : &Env) -> ApTree {
    use Token::*;
    use ApTree::*;
    use ApArity::*;
    match get_arity(&tree) {
        // TOKENS AND VARS
        ZeroAry(V(var)) => reduce_aptree(env.get(&var).unwrap().clone(), &env),

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
            let xz = reduce_aptree(ap(x.clone(), z.clone()), &env);
            let yz = reduce_aptree(ap(y.clone(), z.clone()), &env);
            reduce_aptree(ap(xz, yz), &env)
        },
        Ternary(C, x, y, z) => {
            reduce_aptree(ap(ap(x.clone(), y.clone()), z.clone()), &env)
        },
        Ternary(B, x, y, z) => {
            let yz = reduce_aptree(ap(y.clone(), z.clone()), &env);
            reduce_aptree(ap(x.clone(), yz), &env)
        },
        Unary(I, arg) => (*arg).clone(),
        Ternary(If0, T(Int(0)), x, y) => x.clone(),
        Ternary(If0, T(Int(1)), x, y) => y.clone(),

        // LISTS
        // Cons: TODO: Any action?
        Unary(Car, T(Nil)) => T(True),
        Unary(Car, arg) => {
            match get_arity(&arg) {
                Binary(Cons, car, _)  => car.clone(),
                _ => panic!("Unimplemented: Car as free ap")
            }
        },
        Unary(Cdr, T(Nil)) => T(True),
        Unary(Cdr, arg) => {
            match get_arity(&arg) {
                Binary(Cons, _, cdr)  => cdr.clone(),
                _ => panic!("Unimplemented: Cdr as free ap")
            }
        },
        Binary(Vec, cdr, car) => cons(cdr.clone(), car.clone()),
        // Nil: TODO: Any action?
        Unary(IsNil, T(Nil)) => T(True),
        Unary(IsNil, arg) => {
            match get_arity(&arg) {
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

        

        _ => tree
    }
}


#[derive(Debug)]
pub enum ApPartial {
    PendingBoth,
    PendingRight(ApTree),
    Tree(ApTree),
}

type PartialStack = Vec<ApPartial>;

fn interpret_words(words: &Vec<Word>, env : &Env) -> ApTree {
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
                            top = reduce_aptree(ap(left, top), &env);
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

// Returns the final environment and the last-assigned variable
fn interpret_program(program : &Program) -> (Env, Var) {
    let mut env = Env::default();
    let mut last_var = Var(-100); // Magic?
    for (var, words) in program {
        let expr = interpret_words(words, &env);
        env.insert(*var, expr);
        last_var = *var;
    }
    (env, last_var)
}



#[cfg(test)]
mod test {
    use super::*;
    use ApTree::*;
    use Token::*;
    use Word::*;

    #[test]
    fn test_interpret_words() {
        let env = Env::default();
        assert_eq!(interpret_words(&vec![WT(Int(1))], &env),
                   T(Int(1)));
        assert_eq!(interpret_words(&vec![WAp, WT(Add), WT(Int(1))], &env),
                   Ap(Box::new((T(Add), T(Int(1))))));
        assert_eq!(interpret_words(&vec![WAp, WT(Inc), WT(Int(1))], &env),
                   T(Int(2)));
    }
      
}
