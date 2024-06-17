use std::time::Duration;

use nom::{
    bytes::complete::{tag, take_until, take_while1},
    combinator::map,
    sequence::tuple,
    IResult,
};
use serde::Serialize;

use crate::RconError;

use super::utils::{take_duration, take_u64};

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct GameState {
    pub allied_players: u64,
    pub axis_players: u64,
    pub allied_score: u64,
    pub axis_score: u64,
    pub remaining_time: Duration,
    pub map: String,
    pub next_map: String,
}

impl GameState {
    pub fn parse(input: &str) -> Result<Self, RconError> {
        Ok(take_gamestate(input).map(|o| o.1)?)
    }
}

fn take_gamestate(input: &str) -> IResult<&str, GameState> {
    map(
        tuple((
            tag("Players: Allied: "),
            take_u64,
            tag(" - Axis: "),
            take_u64,
            tag("\nScore: Allied: "),
            take_u64,
            tag(" - Axis: "),
            take_u64,
            tag("\nRemaining Time: "),
            take_duration,
            tag("\nMap: "),
            take_until("\n"),
            tag("\nNext Map: "),
            take_while1(|c| c != '\n'),
        )),
        |(
            _,
            allied_players,
            _,
            axis_players,
            _,
            allied_score,
            _,
            axis_score,
            _,
            remaining_time,
            _,
            map,
            _,
            next_map,
        )| GameState {
            allied_players,
            axis_players,
            allied_score,
            axis_score,
            remaining_time,
            map: map.into(),
            next_map: next_map.into(),
        },
    )(input)
}
