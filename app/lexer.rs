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
    todo!()
}

fn decint(input: &str) -> IResult<&str, &str> {
    n::recognize(n::preceded(n::opt(n::tag("-")), n::digit1))(input)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_variable() {
        assert_eq!(variable(":123"), Ok(("", Var(123))));
    }
}