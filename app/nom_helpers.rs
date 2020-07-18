pub use nom::IResult;
/// This module makes nom's built-in identifiers short while still making it
/// clear they come from nom.
pub mod n {
    pub use nom::branch::*;
    pub use nom::bytes::complete::*;
    pub use nom::character::complete::*;
    pub use nom::combinator::*;
    pub use nom::multi::*;
    pub use nom::sequence::*;
}

pub fn decint(input: &str) -> IResult<&str, &str> {
    n::recognize(n::preceded(n::opt(n::tag("-")), n::digit1))(input)
}
