use std::fmt::Display;

use nom::IResult;
use serde::{Deserialize, Serialize};

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

#[cfg_attr(not(feature = "simple_api"), derive(Serialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum PlayerId {
    /// Steam conventiently uses a u64.
    Steam(u64),

    /// Windows Ids are now, sadly so: an MD5 hash.
    Windows(String),
}

impl Display for PlayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayerId::Steam(i) => i.fmt(f),
            PlayerId::Windows(i) => i.fmt(f),
        }
    }
}

#[cfg(feature = "simple_api")]
impl Serialize for PlayerId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl PlayerId {
    pub fn parse(input: &str) -> Self {
        if let Ok(steam_id) = input.parse() {
            return Self::Steam(steam_id);
        }

        Self::Windows(input.to_string())
    }

    pub fn take(input: &str) -> IResult<&str, PlayerId> {
        Ok(("", Self::parse(input)))
    }
}
