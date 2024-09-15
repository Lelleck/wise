use nom::{
    bytes::complete::{tag, take_until},
    multi::many0,
    IResult,
};
use tracing::error;

use crate::RconError;

use super::{player::PlayerId, Player};

/// Parse a Player from the information returned from the Get PlayerIds command.
///
/// # Parses
/// `1\tPlayer : 11111111111111111\t`
/// `\tPlayer : 11111111111111111\t`
fn take_player(input: &str) -> IResult<&str, Player> {
    let (input, _) = take_until("\t")(input)?;
    let (input, _) = tag("\t")(input)?;
    let (input, encoded_player) = take_until("\t")(input)?;
    let Some(splitter_idx) = encoded_player.rfind(" : ") else {
        error!(encoded_player, "Failed to find ' : ' in player ids");
        return Err(nom::Err::Error(nom::error::Error {
            input,
            code: nom::error::ErrorKind::Eof,
        }));
    };

    let (name, id) = encoded_player.split_at(splitter_idx);
    let (steam_id, _) = tag(" : ")(id)?;
    let id = PlayerId::parse(steam_id);

    let player = Player {
        name: name.to_string(),
        id,
    };

    Ok((input, player))
}

pub fn parse_playerids(input: &str) -> Result<Vec<Player>, RconError> {
    Ok(many0(take_player)(input).map(|o| o.1)?)
}
