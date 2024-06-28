from dataclasses import dataclass
from enum import Enum
from typing import Optional, Union
import json
from types import SimpleNamespace


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
    old: Optional[int]
    new: Optional[int]


@dataclass
class TeamChange:
    """A player switched teams."""
    old: str
    new: str


@dataclass
class RoleChange:
    """A player switched roles."""
    old: str
    new: str


@dataclass
class LoadoutChange:
    """A player equipped another loadout."""
    old: Optional[str]
    new: Optional[str]


@dataclass
class KillsChange:
    """A players kill count changed."""
    old: int
    new: int


@dataclass
class DeathsChange:
    """A players death count changed."""
    old: int
    new: int


@dataclass
class LevelChange:
    """A players level changed."""
    old: int
    new: int


@dataclass
class ScoreChange:
    """The score of a player has changed."""
    type: ScoreType
    old: int
    new: int


PlayerChange = UnitChange | RoleChange | LoadoutChange | TeamChange | KillsChange | DeathsChange | LevelChange | ScoreChange


@dataclass
class PlayerState:
    name: str
    id: str
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
    changes: list[PlayerChange]
    new_state: PlayerState


RconEvent = PlayerEvent
"""Events recorded on the Hell Let Loose server over RCON."""


WiseEvent = RconEvent
"""Any event emitted by wise."""


"""
Parse a JSON emitted by wise. This does not work yet, probably needs custom logic.

For now use the above provided classes to orient yourself how the API works.
"""
def parse_wise_event(obj_str: str) -> WiseEvent:
    # TODO: add parsing logic
    obj = json.loads(obj_str, object_hook=lambda d: SimpleNamespace(**d))
    return obj
    if hasattr(obj, "Rcon"):
        rcon = obj.Rcon

        if hasattr(rcon, "Player"):
            event = rcon.Player
            player = event.player
            player = Player(player.name, extract_id(player.id))

            changes = []
            for change in event.changes:
                changes.append(parse_player_change(change))

            state = PlayerState(**vars(event.new_state))
            state.id = extract_id(state.id)
            return PlayerEvent(player, changes, state)
        
        elif hasattr(rcon, "Log"):
            event = rcon.Log

def parse_player_change(obj: SimpleNamespace) -> any:
    if hasattr(obj, "Score"):
        return ScoreChange(obj.kind, obj.old, obj.new)
    
    (key, value) = get_enum_variant(obj)
    type = globals()[f"{key}Change"]
    return type(**vars(value))

def get_enum_variant(obj: SimpleNamespace) -> tuple[str, SimpleNamespace]:
    key = list(vars(obj).keys())[0]
    value = list(vars(obj).values())[0]
    return (key, value)

def extract_id(obj: SimpleNamespace) -> str:
    return str(obj.Steam) if hasattr(obj, "Steam") else obj.Windows

def is_kind(obj: SimpleNamespace, type: str) -> bool:
    return hasattr(obj, type)

def is_not_kind(obj: SimpleNamespace, type: str) -> bool:
    return not hasattr(obj, type)
