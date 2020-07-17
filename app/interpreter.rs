use crate::aplang::*;
use crate::aplang::ap;

use std::collections::HashMap;

fn reduce_aptree(tree: ApTree, env : &Env) -> ApTree {
    use Token::*;
    use ApTree::*;
    use ApArity::*;
    match get_arity(&tree) {
        ZeroAry(token) => {
            //TODO: Collapse vars
            T(token)
        },

        Unary(Inc, T(Int(n))) => T(Int(n+1)),
        Unary(Dec, T(Int(n))) => T(Int(n-1)),
        Unary(Modulate, T(Int(n))) => { panic!("Unimplemented Modulate") },
        Unary(Demodulate, T(Int(n))) => { panic!("Unimplemented Demodulate") },
        Unary(Neg, T(Int(n))) => T(Int(-n)),
        Unary(Pwr2, T(Int(n))) => { panic!("Unimplemented Pwr2") },
        Unary(IsNil, T(Nil)) => T(True),
        // TODO: Unary(IsNil, Cons) => T(True),

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

type Env = HashMap<Var, ApTree>;

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
