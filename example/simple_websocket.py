import websocket  # Requires the 'websocket-client' library
from colored import Fore, Back, Style
import wise_lib

def on_message(ws, message):
    message = wise_lib.parse_wise_event(message)
    if not hasattr(message, "Rcon"):
        # Thats not supposed to happen, yet!
        return
    
    rcon_event = message.Rcon
    if hasattr(rcon_event, "Player"):
        # Its a player event

        # Get the player this event belongs to
        event_string = ""
        events = []

        player_event = rcon_event.Player
        player = player_event.player
        player_id = player.id.Steam if hasattr(player.id, "Steam") else player.id.Windows
        event_string += f"{player.name}/{Fore.red}{player_id}{Style.reset}"

        changes = player_event.changes
        for change in changes:

            reflection_print(change)
            if hasattr(change, "Score"):
                score_change = change.Score
                events.append(format(Fore.green, score_change.kind, score_change.old, score_change.new))
                continue

            events.append(reflection_print(change))
        
        if not changes:
            events.append(f"{Fore.magenta}{Style.bold}Start polling{Style.reset}")

        buffer = " " * (66 - len(event_string))
        print(f"{event_string}{buffer} | {', '.join(events)}")

def reflection_print(change):
    key = list(change.__dict__.keys())[0]

    color = Fore.blue
    if key in ["Kills", "Deaths"]:
        color = Fore.light_red

    value = list(change.__dict__.values())[0]
    old = value.old
    new = value.new
    return format(color, key, old, new)

def format(color, type, old, new):
    return f"{Style.bold}{color}{type}{Style.reset} {Fore.grey_0}{old}{Style.reset} → {Style.bold}{new}{Style.reset}"

def on_error(ws, error):
    print(f"Error: {error}")

def on_close(ws, close_status_code, close_msg):
    print(f"Connection closed with code: {close_status_code}, message: {close_msg}")

def on_open(ws):
    print("Connection opened")

if __name__ == "__main__":
    websocket_url = "ws://localhost:25052"

    ws = websocket.WebSocketApp(
        websocket_url,
        on_message=on_message,
        on_error=on_error,
        on_close=on_close,
        on_open=on_open
    )

    ws.run_forever()
