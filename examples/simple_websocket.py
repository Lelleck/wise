import websocket  # Requires the 'websocket-client' library
from datetime import datetime
import wise_lib
import sys

ADDRESS = "ws://localhost:25052"
TOKEN = "123"

def on_message(ws, message):
    message = wise_lib.parse_wise_event(message)
    if not hasattr(message, "Rcon"):
        # Thats not supposed to happen, yet!
        return
    
    rcon = message.Rcon
    if hasattr(rcon, "Player"):
        event = rcon.Player
        player = event.player # Every player event belongs to one player

        # The id may be Steam or Windows but in most cases we are only 
        # interested in the string version of it so we can use this helper
        player_id = wise_lib.extract_id(player.id) 
        player_overview = f"{player.name}/{player_id}"

        changes = []
        for change in event.changes:
            # All changes exclusively hold "old" and "new" attributes.
            # Only the ScoreChange also holds its kind which makes it 
            # an exception to the rule.
            if hasattr(change, "Score"):
                change = change.Score
                changes.append(format(change.kind, change.old, change.new))
                continue

            # Many times an object may contain a single key and value.
            # This is the case when attempting to access an enum variant.
            # The below helper function allows us the name and value of the 
            # enum variant inside the object.
            (key, value) = wise_lib.get_enum_variant(change)
            changes.append(format(key, value.old, value.new))
        
        if not changes:
            # If no changes took place Wise has seen this player for the first time
            changes.append(f"Start polling")

        print_prelude("PLAYER")
        print(f"{player_overview} | {', '.join(changes)}")
    
    if hasattr(rcon, "Log"):
        # Its a log event
        print(str(rcon))
        pass

def print_prelude(type):
    current_time = datetime.now().time()
    print(f"{current_time} {type} ", end="")

def format(type, old, new):
    return f"{type} {old} → {new}"

def on_error(ws, error):
    print(f"Error: {error}")

def on_close(ws, code, msg):
    print(f"Connection closed with code: {code}, Message: {msg}")

def on_open(ws):
    if TOKEN:
        ws.send_text(TOKEN)

    print("Connection opened")

if __name__ == "__main__":
    ws = websocket.WebSocketApp(
        ADDRESS,
        on_message=on_message,
        on_error=on_error,
        on_close=on_close,
        on_open=on_open
    )

    ws.run_forever()