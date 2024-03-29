use crate::aplang::*;

use std::fs;
use std::io;
use std::io::BufRead;

use crate::nom_helpers::*;

pub fn lex(file: &str) -> Program {
    let mut program: Program = vec![];

    let file = fs::File::open(file).unwrap();
    let lines = io::BufReader::new(file).lines();
    for line in lines {
        let line = line.unwrap();
        let (remaining, assignment) =
            assignment(&line).expect("Parse error (TODO: error handling)");
        assert_eq!(remaining, "");
        program.push(assignment);
    }
    program
}

pub fn oneliner(expr: &str) -> Program {
    let (_, words) = crate::lexer::aplist(&(String::from(" ") + expr)).unwrap();
    vec![(Var(-1), words)]
}

/// Doesn't include the newline
fn assignment(line: &str) -> IResult<&str, Assignment> {
    n::separated_pair(variable, n::tag(" ="), aplist)(line)
}


fn variable(s: &str) -> IResult<&str, Var> {
    n::map(
        n::alt((
            n::map_res(n::preceded(n::tag(":"), decint), |s: &str| s.parse()),
            n::value(-1, n::tag("galaxy")),
        )),
        Var,
    )(s)
}

pub fn aplist(s: &str) -> IResult<&str, Vec<Word>> {
    n::many1(n::preceded(n::tag(" "), word))(s)
}

fn token(input: &str) -> IResult<&str, Token> {
    use Token::*;

    // In galaxy.txt: add ap b c car cdr cons div eq i isnil lt mul neg nil s t

    n::alt((
        n::map(n::map_res(decint, |s: &str| s.parse()), Int),
        n::value(If0, n::tag("if0")),
        n::value(Add, n::tag("add")),
        n::value(Inc, n::tag("inc")),
        n::value(Car, n::tag("car")),
        n::value(Cdr, n::tag("cdr")),
        n::value(Cons, n::tag("cons")),
        n::value(Div, n::tag("div")),
        n::value(Eq, n::tag("eq")),
        n::value(IsNil, n::tag("isnil")),
        n::value(Lt, n::tag("lt")),
        n::value(Multiply, n::tag("mul")),
        n::value(Neg, n::tag("neg")),
        n::value(Nil, n::tag("nil")),
        n::value(B, n::tag("b")),
        n::value(C, n::tag("c")),
        n::value(False, n::tag("f")),
        n::value(I, n::tag("i")),
        n::value(S, n::tag("s")),
        n::value(True, n::tag("t")),
        n::map(variable, V),
    ))(input)
}

fn word(input: &str) -> IResult<&str, Word> {
    n::alt((n::map(token, Word::WT), n::value(Word::WAp, n::tag("ap"))))(input)
}

#[cfg(test)]
mod test {
    use super::*;
    use Token::*;
    use Word::*;

    #[test]
    fn test_variable() {
        assert_eq!(variable(":123"), Ok(("", Var(123))));
    }

    #[test]
    fn test_word() {
        assert_eq!(word("ap"), Ok(("", WAp)));
        assert_eq!(word("cons"), Ok(("", WT(Cons))));
        assert_eq!(word("nil"), Ok(("", WT(Nil))));
        assert_eq!(word("123"), Ok(("", WT(Int(123.into())))));
        assert_eq!(word("-111"), Ok(("", WT(Int((-111).into())))));
        assert_eq!(word("560803991675135"), Ok(("", WT(Int(560803991675135i64.into())))));
    }

    #[test]
    fn test_assignment() {
        assert_eq!(assignment(":1 = 2"), Ok(("", (Var(1), vec![WT(Int(2.into()))]))));
        assert_eq!(
            assignment("galaxy = 42"),
            Ok(("", (Var(-1), vec![WT(Int(42.into()))])))
        );
        assert_eq!(
            assignment(":2 = ap i :1"),
            Ok(("", (Var(2), vec![WAp, WT(I), WT(V(Var(1)))])))
        );
        assert_eq!(
            assignment(":1030 = ap ap cons 2 ap ap cons 7 nil"),
            Ok((
                "",
                (
                    Var(1030),
                    vec![
                        WAp,
                        WAp,
                        WT(Cons),
                        WT(Int(2.into())),
                        WAp,
                        WAp,
                        WT(Cons),
                        WT(Int(7.into())),
                        WT(Nil),
                    ]
                )
            ))
        );
    }
}
