// Lexer types

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Var (pub i32);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Token {
    // Constants
    True,
    False,
    Int(i64), //TODO: BigInt?
    V(Var),
    // Unary operators
    Inc,
    Dec,
    Neg,
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
    Nil,   // We pretend it is a token (crossing fingers), but lambda def is: nil x = t
    IsNil,
    // Drawing
    Draw, // draw list[<vecs-of pixels>] = screen with listed pixels filled
    Checkerboard, // = chkb n x = screen with n*n checkered pattern in upper-left, if x=0.
    DrawList, //TODO: Word. drawlist ([ x,y,... ]) = [ draw x, draw y, ... ]
    // Protocol
    Send,
    Modulate,
    Demodulate,
}

pub fn is_eager_fun1(fun: &Token) -> bool {
    use Token::*;
    match fun {
        Inc | Dec | Neg | Pwr2 | Car | Cdr | IsNil | I => true,
        _ => false,
    }
}

pub fn is_eager_fun2(fun: &Token) -> bool {
    use Token::*;
    match fun {
        Add | Multiply | Div | Eq | Lt => true,
        _ => false,
    }
}

// fn is_eager_fun2(fun: Token) {
//     match fun {
//         [Inc, Dec, Neg, Pwr2, Add, Multiply, Div, Eq, Lt, If0, Car, Cdr, IsNil] => true,
//         _ => false,
//     }
// }

// fn numerical_fun(fun: Token) {
//     //TODO
//     match fun {
//         [Inc, Dec] => true,
//         _ => false,
//     }
// }

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Word {
    WAp,
    // OpenList,
    // ListSep,
    // CloseList,
    WT(Token),
}


pub type Assignment = (Var, Vec<Word>);

pub type Program = Vec<Assignment>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApTree {
    Ap(Box<(ApTree,ApTree)>),
    T(Token),
    // List(Vec<ApTree>)
}

pub enum ApArity<'a> {
    ZeroAry(Token),
    Unary(Token, &'a ApTree),
    Binary(Token, &'a ApTree, &'a ApTree),
    Ternary(Token, &'a ApTree, &'a ApTree, &'a ApTree),
    TooManyAry(&'a ApTree, &'a ApTree),
}

pub fn ap(arg1: ApTree, arg2: ApTree) -> ApTree {
    return ApTree::Ap(Box::from((arg1, arg2)));
}

pub fn nil() -> ApTree {
    return ApTree::T(Token::Nil);
}

pub fn cons(head: ApTree, tail: ApTree) -> ApTree {
    return ap(ap(ApTree::T(Token::Cons), head), tail);
}

pub fn int(val: i64) -> ApTree {
    return ApTree::T(Token::Int(val));
}
