from dataclasses import dataclass
from enum import Enum
from typing import Optional, Union
from uuid import UUID
import json
from types import SimpleNamespace

@dataclass
class SteamId:
    id: int


@dataclass
class WindowsId:
    """"""
    id: UUID


PlayerId = SteamId | WindowsId
"""Any type of id used to identify players."""


@dataclass
class Player:
    """Represents a player in the game."""
    name: str
    id: str


class ScoreType(Enum):
    """Different types of score that can update."""
    COMBAT = 1
    OFFENSE = 2
    DEFENSE = 3
    SUPPORT = 4


@dataclass
class UnitChange:
    """A player switched units."""
    old_unit: Optional[int]
    new_unit: Optional[int]


@dataclass
class TeamChange:
    """A player switched teams."""
    old_team: str
    new_team: str


@dataclass
class RoleChange:
    """A player switched roles."""
    old_role: str
    new_role: str


@dataclass
class LoadoutChange:
    """A player equipped another loadout."""
    old_loadout: Optional[str]
    new_loadout: Optional[str]


@dataclass
class KillsChange:
    """A players kill count changed."""
    old_kills: int
    new_kills: int


@dataclass
class DeathsChange:
    """A players death count changed."""
    old_deaths: int
    new_deaths: int


@dataclass
class LevelChange:
    """A players level changed."""
    old_level: int
    new_level: int


@dataclass
class ScoreChange:
    """The score of a player has changed."""
    type: ScoreType
    old_score: int
    new_score: int


PlayerChange = UnitChange | RoleChange | LoadoutChange | TeamChange | KillsChange | DeathsChange | LevelChange | ScoreChange


@dataclass
class PlayerState:
    name: str
    id: PlayerId 
    team: str
    role: str
    unit: Optional[int]
    loadout: Optional[str]
    kills: int
    deaths: int
    combat_score: int
    offense_score: int
    defense_score: int
    support_score: int
    level: int


@dataclass
class PlayerEvent:
    """Represents an event related to a player."""
    player: Player
    changes: PlayerChange
    new_state: PlayerState


RconEvent = PlayerEvent
"""Events recorded on the Hell Let Loose server over RCON."""


WiseEvent = RconEvent
"""Any event emitted by wise."""


"""
Parse a JSON emitted by wise. This does not work yet, probably needs custom logic.

For now use the above provided classes to orient yourself how the API works.
"""
def parse_wise_event(obj_str: str):
    # TODO: add parsing logic
    return json.loads(obj_str, object_hook=lambda d: SimpleNamespace(**d))
