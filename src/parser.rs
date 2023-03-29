use anyhow::{Error, Result};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1, line_ending, not_line_ending, one_of, space0},
    combinator::{map, map_res, recognize},
    multi::{length_count, many0, many1},
    sequence::{delimited, separated_pair, terminated, tuple},
    IResult,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    default::default,
    str::{self, FromStr},
};
use tracing::trace;

pub fn parse(input: &str) -> Result<Parsed, nom::Err<nom::error::Error<&str>>> {
    let mut output: Parsed = default();
    let (input, _) = tuple((
        map(line(tag("Name"), not_line_ending), |name| {
            trace!(%name);
            output.name = name.to_owned()
        }),
        many0(alt((
            map(line(tag("Comments"), not_line_ending), |comments| {
                trace!(%comments);
                output.comments = comments.to_owned()
            }),
            map(line(tag("CAS#"), number), |cas| {
                trace!(%cas);
                output.cas = Some(cas)
            }),
            map(line(tag("DB#"), number), |db| {
                trace!(%db);
                output.db = Some(db)
            }),
            map(line(tag("Formula"), not_line_ending), |formula| {
                trace!(%formula);
                output.formula = formula.to_owned()
            }),
            map(line(tag("MW"), number), |mw| {
                trace!(%mw);
                output.mw = Some(mw)
            }),
            map(line(tag("NIST#"), number), |nist| {
                trace!(%nist);
                output.nist = Some(nist)
            }),
            map(line(tag("Synonym"), not_line_ending), |synonym| {
                trace!(%synonym);
                output.synonym = synonym.to_owned()
            }),
        ))),
        map(peaks, |peaks| {
            trace!(?peaks);
            output.peaks = peaks.into_iter().collect()
        }),
    ))(input)?;
    assert!(input.is_empty());
    Ok(output)
}

fn line<'a, T>(
    tag: impl Fn(&'a str) -> IResult<&'a str, &'a str>,
    value: impl FnMut(&'a str) -> IResult<&'a str, T>,
) -> impl FnMut(&'a str) -> IResult<&str, T> {
    delimited(
        terminated(tag, char(':')),
        delimited(space0, value, space0),
        line_ending,
    )
}

fn peaks(input: &str) -> IResult<&str, Vec<(u64, u64)>> {
    length_count(
        line(tag("Num Peaks"), number::<u64>),
        terminated(
            separated_pair(number, multiseparator, number),
            multiseparator,
        ),
    )(input)
}

fn multiseparator(input: &str) -> IResult<&str, Vec<&str>> {
    many1(alt((recognize(one_of(" \t,;:()[]{}")), line_ending)))(input)
}

fn number<T: FromStr>(input: &str) -> IResult<&str, T> {
    map_res(digit1, str::parse)(input)
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Parsed {
    pub name: String,
    pub cas: Option<u64>,
    pub comments: String,
    pub db: Option<u64>,
    pub formula: String,
    pub mw: Option<u64>,
    pub nist: Option<u64>,
    pub synonym: String,
    pub peaks: BTreeMap<u64, u64>,
}

impl Parsed {
    pub fn intensities(&self) -> Vec<u64> {
        let mut intensities = Vec::new();
        for (&mass, &intensity) in &self.peaks {
            while mass > intensities.len() as _ {
                intensities.push(0);
            }
            intensities.push(intensity);
        }
        intensities
    }
}

impl FromStr for Parsed {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(parse(value).map_err(|error| error.to_owned())?)
    }
}
