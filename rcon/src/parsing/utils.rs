use std::num::ParseIntError;

use nom::{
    bytes::complete::tag, character::complete::digit1, combinator::map, sequence::tuple, IResult,
};

pub fn parse_u64(input: &str) -> Result<u64, ParseIntError> {
    input.parse()
}

pub fn take_u64(input: &str) -> IResult<&str, u64> {
    let (input, num_str) = digit1(input)?;
    let num = num_str.parse().map_err(|_| {
        nom::Err::Failure(nom::error::Error::new(input, nom::error::ErrorKind::Digit))
    })?;
    Ok((input, num))
}

pub fn take_duration(input: &str) -> IResult<&str, u64> {
    map(
        tuple((take_u64, tag(":"), take_u64, tag(":"), take_u64)),
        |(hours, _, minutes, _, seconds)| hours * 3600 + minutes * 60 + seconds,
    )(input)
}
