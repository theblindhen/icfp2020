// Lexer types
use std::collections::HashMap;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Var (i32);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Token {
    // Constants
    True,
    False,
    Int(i32), //TODO: BigInt?
    Var(Var),
    // Unary operators
    Inc,
    Dec,
    Modulate,
    Demodulate,
    Pwr2,
    // Binary operators
    Add,
    Multiply,
    Div, // integer division
    // Comparison on integers
    Eq,
    Lt,
    // Combinators
    S, // S x y z = x z (y z)
    C, // C x y z = x z y
    B, // B x y z = x (y z)
    I, // identity
    If0, // if0 0 x y = x    and    if0 1 x y = y
    // Lists
    Cons,  // cons x y z   =  (z x) y
    Car,   // car (cons x y) = x    and    car x = x t
    Cdr,   // cdr (cons x y) = y    and    cdr x = x f
    Vec,   // Alias for Cons, perhaps meant for pairs-of-ints (pixels)
    Nil,   // nil x = t
    IsNil,
    // Drawing
    Draw, // draw list[<vecs-of pixels>] = screen with listed pixels filled
    Checkerboard, // = chkb n x = screen with n*n checkered pattern in upper-left, if x=0.
    DrawList, //TODO: Word. drawlist ([ x,y,... ]) = [ draw x, draw y, ... ]
    // Protocol
    Send,

}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Word {
    Ap,
    // OpenList,
    // ListSep,
    // CloseList,
    Token(Token),
}


pub type Assignment = (Var, Vec<Word>);

pub type Program = Vec<Assignment>;


// Interpreter types

#[derive(Debug, PartialEq, Eq)]
pub enum ApTree {
    Ap(Box<(ApTree,ApTree)>),
    T(Token),
    // List(Vec<ApTree>)
}

fn reduce_aptree(tree: ApTree, env : &Env) -> ApTree {
    use Token::*;
    use ApTree::*;
    match &tree {
        Ap(ap_arms) => {
            match **ap_arms {
                // Unary Ops
                (T(Inc), T(Int(n))) => T(Int(n+1)),
                _ => tree
            }
        },
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
    use ApPartial::*;
    let mut stack = PartialStack::default();
    for token in words {
        match token {
            Word::Ap => stack.push(PendingBoth),
            Word::Token(t) => {
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
                            top = reduce_aptree(ApTree::Ap(Box::new((left, top))), &env);
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

    #[test]
    fn test_interpret_words() {
        let env = Env::default();
        assert_eq!(interpret_words(&vec![Word::Token(Int(1))], &env),
                   T(Int(1)));
        assert_eq!(interpret_words(&vec![Word::Ap, Word::Token(Add), Word::Token(Int(1))], &env),
                   Ap(Box::new((T(Add), T(Int(1))))));
    }
      
}
