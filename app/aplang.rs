// Lexer types

struct Var (i32);

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

pub enum ApTree {
    Ap(Box<(ApTree,ApTree)>),
    Token(Token),
    // List(Vec<ApTree>)
}

pub enum ApPartial {
    PendingBoth,
    PendingRight(ApTree)
}

pub type ApStack = Vec<ApPartial>;

type Env = Map<Var, ApTree>;

fn interpret_expr(expr: Vec<Words>) -> ApTree {
    for 

}

// Returns the final environment and the last-assigned variable
fn interpret_program(program : &Program) -> (Env, Var) {
    let mut env = Env::default();
    let mut last_var = -100; // Magic?
    for (var, words) in program {
        let expr = interpret_expr(words);
        env[var] = expr;
        last_var = var;
    }
}
