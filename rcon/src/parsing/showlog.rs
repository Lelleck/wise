use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{char, multispace0},
    combinator::recognize,
    error::{Error, ErrorKind},
    multi::many0,
    sequence::{delimited, separated_pair, tuple},
    Err, IResult,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::{error, trace};

use crate::RconError;

use super::{utils::take_u64, Player, PlayerId};

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct LogLine {
    pub timestamp: u64,
    pub kind: LogKind,
}

//        [38:03 min (1718194470)] MATCH ENDED `CARENTAN WARFARE` ALLIED (2 - 2) AXIS
//        [36:18 min (1718194575)] MATCH START SAINTE-MÈRE-ÉGLISE WARFARE
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum LogKind {
    Connect {
        player: Player,
        has_connected: bool,
    },
    TeamSwitch {
        player: Player,
        old_team: String,
        new_team: String,
    },
    Kill {
        killer: Player,
        killer_faction: String,
        victim: Player,
        victim_faction: String,
        is_teamkill: bool,
        weapon: String,
    },
    MatchStart {
        map: String,
    },
    MatchEnded {
        map: String,
        allied_score: u64,
        axis_score: u64,
    },
    Chat {
        sender: Player,
        team: String,
        reach: String,
        content: String,
    },
}

/// Parse the prelude of every log message and extract the timestamp.
///
/// # Parses
/// `[44.7 sec (1718212472)] `
fn take_prelude(input: &str) -> IResult<&str, u64> {
    let (input, timestamp) = delimited(
        char('['),
        tuple((take_until("("), char('('), take_u64, char(')'))),
        tag("] "),
    )(input)?;

    Ok((input, timestamp.2))
}

/// Parse a connect or disconnect log line.
///
/// # Parses
/// `CONNECTED PlayerName (11111111111111111)`
/// `DISCONNECTED PlayerName (11111111111111111)`
fn take_connect(input: &str) -> IResult<&str, LogKind> {
    let (input, connect) = alt((tag("CONNECTED"), tag("DISCONNECTED")))(input)?;
    let connect = connect == "CONNECTED";

    let (input, name_and_id) = take_until("\n")(input)?;
    let Some(space_idx) = name_and_id.rfind(" ") else {
        error!(name_and_id, "Failed to rfind ' ' in connect log line");
        return Err(Err::Failure(Error::new(input, ErrorKind::Space)));
    };
    let (name, id) = name_and_id.split_at(space_idx);
    let (name, _) = tag(" ")(name)?;
    let id = &id[2..id.len() - 1];
    let id = PlayerId::parse(id);
    let player = Player::new(name.to_string(), id);
    return Ok((
        input,
        LogKind::Connect {
            player,
            has_connected: connect,
        },
    ));
}

/// Parse a kill or team kill log line.
///
/// # Security
///
/// The parsing of this function is entirely based upon the idea that usernames are capped at a length of 17 characters.
/// Otherwise users would be able to inject a " -> " into their name along with a valid name and faction info.
///
/// # Parses
/// `KILL: Player Name(Allies/11111111111111111) -> PlayerName(Axis/11111111111111111) with M1903 SPRINGFIELD`
/// `TEAM KILL: Player Name(Axis/11111111-aaaa-1111-aaaa-111111111111) -> PlayerName(Axis/11111111111111111) with Opel Blitz (Transport)`
fn take_kill(input: &str) -> IResult<&str, LogKind> {
    let (input, kill_type) = alt((tag("KILL: "), tag("TEAM KILL: ")))(input)?;
    let is_teamkill = kill_type == "TEAM KILL: ";

    let (input, killer_arrow_victim_with_weapon) = take_until("\n")(input)?;
    let Some(with_idx) = killer_arrow_victim_with_weapon.rfind(" with ") else {
        error!(
            killer_arrow_victim_with_weapon,
            "Failed to rfind 'with' in kill log line"
        );
        return Err(Err::Error(Error::new(
            killer_arrow_victim_with_weapon,
            ErrorKind::Fail,
        )));
    };

    let (killer_arrow_victim, with_weapon) = killer_arrow_victim_with_weapon.split_at(with_idx);
    let (weapon, _) = tag(" with ")(with_weapon)?;

    let (arrow_victim, (killer_faction, killer)) = take_faction_and_player(killer_arrow_victim)?;
    let (only_victim, _arrow) = tag(" -> ")(arrow_victim)?;
    let (_, (victim_faction, victim)) = take_faction_and_player(only_victim)?;

    let kind = LogKind::Kill {
        killer,
        killer_faction,
        victim,
        victim_faction,
        is_teamkill,
        weapon: weapon.to_string(),
    };

    return Ok((input, kind));
}

/// Take a match start/end log line.
///
/// ```
/// use rcon::parsing::showlog::*;
///
/// let (_, log) = take_match("MATCH START         MATCH START SAINTE-MÈRE-ÉGLISE WARFARE\n").unwrap();
/// assert!(matches!(log, LogKind::MatchStart { .. }));
///
/// let (_, log) = take_match("MATCH ENDED         MATCH ENDED `SAINTE-MÈRE-ÉGLISE WARFARE` ALLIED (2 - 2) AXIS\n").unwrap();
/// assert!(matches!(log, LogKind::MatchEnded { allied_score: 2, axis_score: 2, .. }));
/// ```
pub fn take_match(input: &str) -> IResult<&str, LogKind> {
    let (input, _) = tag("MATCH ")(input)?;
    let (input, kind) = alt((tag("START"), tag("ENDED")))(input)?;
    let (remaining_logs, space_map_info) = take_until("\n")(input)?;
    let (map_info, _) = multispace0(space_map_info)?;

    if kind == "START" {
        return Ok((
            remaining_logs,
            LogKind::MatchStart {
                map: map_info.to_string(),
            },
        ));
    }

    if kind != "ENDED" {
        error!(
            map_info,
            "Match log line is neither of kind `START` nor `ENDED` instead `{}`", kind
        );
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            ErrorKind::Fail,
        )));
    }

    let log = take_match_ended(input).unwrap();
    Ok((remaining_logs, log))
}

/// Takes a match ended value and parses it using
fn take_match_ended(input: &str) -> Result<LogKind, Option<regex::Error>> {
    let pattern = r"`([^*]*)`.*\((\d+)\s*-\s*(\d+)\)";
    let re = Regex::new(&pattern)?;

    let Some(caps) = re.captures(input) else {
        error!("Failed to capture match ended info for `{}`", input);
        return Err(None);
    };

    if caps.len() != 4 {
        error!("Failed to capture 3 groups for `{input}`");
        return Err(None);
    }

    let map_name = &caps[1];
    let Ok(allied_score) = caps[2].parse() else {
        return Err(None);
    };

    let Ok(axis_score) = caps[3].parse() else {
        return Err(None);
    };

    Ok(LogKind::MatchEnded {
        map: map_name.to_string(),
        allied_score,
        axis_score,
    })
}

/// Parse the name, faction and id component of a kill log.
///
/// # Parses
/// `Player Name(Allies/11111111111111111)`
/// `Player Name(Axis/11111111-aaaa-1111-aaaa-111111111111)`
fn take_faction_and_player(input: &str) -> IResult<&str, (String, Player)> {
    // Watch out and handle UTF-8 correctly.
    if input.chars().count() < 27 {
        return Err(Err::Failure(Error::new(input, ErrorKind::Eof)));
    }

    let mut end = 0;
    while end < 20 {
        let (idx, _) = input.char_indices().nth(end).unwrap();
        let (name, faction_id) = input.split_at(idx);
        let reco = recognize(take_faction_and_id)(faction_id);

        if reco.is_err() {
            end += 1;
            continue;
        }

        let (input, faction_id) = reco.unwrap();
        let (_, (faction, id)) = take_faction_and_id(faction_id)?;
        return Ok((input, (faction, Player::new(name.to_string(), id))));
    }

    Err(Err::Error(Error::new(input, ErrorKind::Eof)))
}

/// Parse the faction and id component of a kill log.
///
/// # Parses
/// `(Allies/11111111111111111)`
/// `(Axis/11111111-aaaa-1111-aaaa-111111111111)`
fn take_faction_and_id(original_input: &str) -> IResult<&str, (String, PlayerId)> {
    let (input, (faction, player_id)) = delimited(
        char('('),
        separated_pair(take_while1(|c| c != '/'), char('/'), take_until(")")),
        char(')'),
    )(original_input)?;

    if player_id.len() < 17 {
        return Err(Err::Error(Error::new(
            original_input,
            ErrorKind::LengthValue,
        )));
    }
    let (_, player_id) = PlayerId::take(player_id)?;
    Ok((input, (faction.to_string(), player_id)))
}

/// Parse a chat message.
///
/// # Parses
/// `CHAT[Team][Player(Allies/11111111111111111)]: foo bar`
fn take_chat(input: &str) -> IResult<&str, LogKind> {
    let (input, _) = tag("CHAT")(input)?;
    let (input, reach) = delimited(char('['), take_while1(|c| c != ']'), char(']'))(input)?;
    let (input, _) = tag("[")(input)?;
    let (input, (team, sender)) = take_faction_and_player(input)?;
    let (input, _) = tag("]: ")(input)?;
    let (input, content) = take_until("\n")(input)?;

    Ok((
        input,
        LogKind::Chat {
            sender,
            team,
            reach: reach.to_string(),
            content: content.to_string(),
        },
    ))
}

/// Parse an entire log line if possible otherwise skip until the next "\n".
fn take_logline(input: &str) -> IResult<&str, Option<LogLine>> {
    let res = take_prelude(input);

    // If parsing the prelude fails skip this line, such as the case with multi-line messages
    if let Err(_) = res {
        let (input, _) = tuple((take_until("\n"), tag("\n")))(input)?;
        return Ok((input, None));
    }
    let (input, timestamp) = res.unwrap();

    // If we fail to parse the log line skip it
    let Ok((input, kind)) = alt((take_connect, take_kill, take_chat, take_match))(input) else {
        let (input, skipped) = tuple((take_until("\n"), tag("\n")))(input)?;
        trace!("Failed to parse `{}`", skipped.0);

        return Ok((input, None));
    };
    let (input, _) = tag("\n")(input)?;
    return Ok((input, Some(LogLine { timestamp, kind })));
}

pub fn parse_loglines(input: &str) -> Result<Vec<LogLine>, RconError> {
    Ok(many0(take_logline)(input).map(|o| {
        o.1.into_iter()
            .filter(|p| p.is_some())
            .map(|p| p.unwrap())
            .collect()
    })?)
}
