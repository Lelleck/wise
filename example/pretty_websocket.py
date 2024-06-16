import websocket  # Requires the 'websocket-client' library
from colored import Fore, Back, Style  # Requires the 'colored' library
from datetime import datetime
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
        event_string += f"{player.name} {Fore.grey_0}{player_id}{Style.reset}"

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
        print_prelude("PLAYER")
        print(f"{event_string}{buffer} | {', '.join(events)}")
    
    if hasattr(rcon_event, "Log"):
        print_prelude("LOG")

        event_string = f"{Style.bold}{Fore.magenta}Unknown event{Style.reset}"
        log_event = rcon_event.Log
        if hasattr(log_event.kind, "Kill"):
            kill_event = log_event.kind.Kill
            killer_id = get_id(kill_event.killer.id)
            victim_id = get_id(kill_event.victim.id)
            type = "team kills" if kill_event.is_teamkill else "kills"
            event_string = f"({kill_event.killer_faction}) {kill_event.killer.name} {Fore.grey_0}{killer_id}{Style.reset} {Style.bold}{Fore.light_red}{type}{Style.reset} ({kill_event.victim_faction}) {kill_event.victim.name} {Fore.grey_0}{victim_id}{Style.reset} with {Style.bold}{kill_event.weapon}{Style.reset}"

        if hasattr(log_event.kind, "Connect"):
            connect_event = log_event.kind.Connect
            player_id = get_id(connect_event.player.id)
            type = "connects" if connect_event.connect else "disconnects"
            event_string = f"{connect_event.player.name} {Fore.grey_0}{player_id}{Style.reset} {Style.bold}{Back.light_blue} {type} {Style.reset}"
            pass

        print(event_string)
        pass

def get_id(obj) -> str:
    return obj.Steam if hasattr(obj, "Steam") else obj.Windows

def reflection_print(change):
    key = list(change.__dict__.keys())[0]

    color = Fore.blue
    if key in ["Kills", "Deaths"]:
        color = Fore.light_red

    value = list(change.__dict__.values())[0]
    old = value.old
    new = value.new
    return format(color, key, old, new)

def print_prelude(type):
    back = None
    fore = Fore.black 
    if type == "PLAYER":
        back = Back.light_green
    elif type == "LOG":
        back = Back.light_blue
        type = "LOG   "
    elif type == "MATCH":
        back = Back.light_red
        fore = Fore.white
    
    current_time = datetime.now().time()
    fmt_time = current_time.strftime("%H:%M:%S")
    print(f"{back} {fmt_time}{Style.bold}{fore} - {type} {Style.reset} ", end="")

def format(color, type, old, new):
    return f"{Style.bold}{color}{type}{Style.reset} {Fore.grey_0}{old}{Style.reset} → {Style.bold}{new}{Style.reset}"

def on_error(ws, error):
    print(f"Error: {error}")

def on_close(ws, close_status_code, close_msg):
    print(f"Connection closed with code: {close_status_code}, message: {close_msg}")

def on_open(ws):
    print(f"{Back.magenta}{Style.bold} Connection opened {Style.reset}")

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
