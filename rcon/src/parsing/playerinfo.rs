use super::{utils::*, PlayerId};
use crate::RconError;
use nom::{
    bytes::complete::{tag, take_until},
    character::complete::digit1,
    combinator::{map, map_res, opt},
    sequence::tuple,
    IResult,
};
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PlayerInfo {
    pub name: String,
    pub id: PlayerId,
    // TODO: represent the "None" team as None to preserve consistency?
    pub team: String,
    pub role: String,
    pub unit: Option<u64>,
    pub loadout: Option<String>,
    pub kills: u64,
    pub deaths: u64,
    pub combat_score: u64,
    pub offense_score: u64,
    pub defense_score: u64,
    pub support_score: u64,
    pub level: u64,
}

impl PlayerInfo {
    pub fn parse(input: &str) -> Result<Option<Self>, RconError> {
        if input == "FAIL" {
            return Ok(None);
        }

        Ok(Some(take_playerinfo(input).map(|o| o.1)?))
    }
}

/// Parses scores.
fn take_score(input: &str) -> IResult<&str, (u64, u64, u64, u64)> {
    // Made to parse:
    //C 0, O 0, D 0, S 0
    return map(
        tuple((
            tag("C "),
            map_res(digit1, parse_u64),
            tag(", O "),
            map_res(digit1, parse_u64),
            tag(", D "),
            map_res(digit1, parse_u64),
            tag(", S "),
            map_res(digit1, parse_u64),
        )),
        |(_, combat, _, offense, _, defense, _, support)| (combat, offense, defense, support),
    )(input);
}

fn take_playerinfo(input: &str) -> IResult<&str, PlayerInfo> {
    return map(
        tuple((
            tag("Name: "),
            take_until("\n"),
            tag("\nsteamID64: "),
            map_res(take_until("\n"), PlayerId::take),
            tag("\nTeam: "),
            take_until("\n"),
            tag("\nRole: "),
            take_until("\n"),
            opt(tuple((
                tag("\nUnit: "),
                map_res(digit1, parse_u64),
                take_until("\n"),
            ))),
            opt(tuple((tag("\nLoadout: "), take_until("\n")))),
            tag("\nKills: "),
            map_res(digit1, parse_u64),
            tag(" - Deaths: "),
            map_res(digit1, parse_u64),
            tag("\nScore: "),
            take_score,
            tag("\nLevel: "),
            map_res(digit1, parse_u64),
        )),
        |(
            _,
            name,
            _,
            id,
            _,
            team,
            _,
            role,
            unit,
            loadout,
            _,
            kills,
            _,
            deaths,
            _,
            (combat_score, offense_score, defense_score, support_score),
            _,
            level,
        )| PlayerInfo {
            name: name.into(),
            id: id.1,
            team: team.into(),
            role: role.into(),
            loadout: loadout.map(|s| s.1.into()),
            unit: unit.map(|u| u.1),
            kills,
            deaths,
            combat_score,
            offense_score,
            defense_score,
            support_score,
            level,
        },
    )(input);
}
