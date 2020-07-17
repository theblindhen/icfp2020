use crate::aplang::*;

use std::fs;
use std::io;
use std::io::BufRead;

use nom::IResult;
/// This module makes nom's built-in identifiers short while still making it
/// clear they come from nom.
mod n {
    pub use nom::branch::*;
    pub use nom::bytes::complete::*;
    pub use nom::character::complete::*;
    pub use nom::combinator::*;
    pub use nom::multi::*;
    pub use nom::sequence::*;
}

pub fn lex(file: &str) -> Result<Program, Box<dyn std::error::Error>> {
    let mut program: Program = vec![];

    let file = fs::File::open(file)?;
    let lines = io::BufReader::new(file).lines();
    for line in lines {
        let line = line?;
        let (remaining, assignment) =
            assignment(&line).expect("Parse error (TODO: error handling)");
        assert_eq!(remaining, "");
        program.push(assignment);
    }
    Ok(program)
}

/// Doesn't include the newline
fn assignment(line: &str) -> IResult<&str, Assignment> {
    n::separated_pair(variable, n::tag(" ="), aplist)(line)
}

fn variable(s: &str) -> IResult<&str, Var> {
    n::map(
        n::map_res(n::preceded(n::tag(":"), decint), |s: &str| s.parse()),
        Var,
    )(s)
}

fn aplist(s: &str) -> IResult<&str, Vec<Word>> {
    n::many1(n::preceded(n::tag(" "), word))(s)
}

fn decint(input: &str) -> IResult<&str, &str> {
    n::recognize(n::preceded(n::opt(n::tag("-")), n::digit1))(input)
}

fn token(input: &str) -> IResult<&str, Token> {
    use Token::*;

    n::alt((
        n::map(n::map_res(decint, |s: &str| s.parse()), Int),
        n::value(Cons, n::tag("cons")),
        n::value(Nil, n::tag("nil")),
    ))(input)
}

fn word(input: &str) -> IResult<&str, Word> {
    n::alt((n::map(token, Word::WT), n::value(Word::WAp, n::tag("ap"))))(input)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::aplang::Var;
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
        assert_eq!(word("123"), Ok(("", WT(Int(123)))));
        assert_eq!(word("-111"), Ok(("", WT(Int(-111)))));
    }

    #[test]
    fn test_assignment() {
        assert_eq!(assignment(":1 = 2"), Ok(("", (Var(1), vec![WT(Int(2))]))));
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
                        WT(Int(2)),
                        WAp,
                        WAp,
                        WT(Cons),
                        WT(Int(7)),
                        WT(Nil),
                    ]
                )
            ))
        );
    }
}
