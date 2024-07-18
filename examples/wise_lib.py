import json
from types import SimpleNamespace

"""Convert a string into a simple namespace."""
def json_into_namespace(obj_str: str) -> SimpleNamespace:
    # TODO: add parsing logic
    obj = json.loads(obj_str, object_hook=lambda d: SimpleNamespace(**d))
    return obj

"""Given a SimpleNamespace get the first key and value."""
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
