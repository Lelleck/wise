use nom::combinator::map;
use nom::{branch::alt, IResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::utils::{take_u64, take_uuid};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub id: PlayerId,
}

impl Player {
    pub fn new(name: String, id: PlayerId) -> Self {
        Self { name, id }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlayerId {
    Steam(u64),
    Windows(Uuid),
}

impl PlayerId {
    pub fn parse(input: &str) -> Option<Self> {
        if let Ok(steam_id) = input.parse() {
            return Some(Self::Steam(steam_id));
        }

        if let Ok(uuid) = Uuid::parse_str(input) {
            return Some(Self::Windows(uuid));
        }

        None
    }

    pub fn take(input: &str) -> IResult<&str, Self> {
        // Make sure that UUID check comes first as the u64 check may simple consume the
        // first digit of the UUID.
        alt((
            map(take_uuid, PlayerId::Windows),
            map(take_u64, PlayerId::Steam),
        ))(input)
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Steam(i) => i.to_string(),
            Self::Windows(i) => i.to_string(),
        }
    }
}
