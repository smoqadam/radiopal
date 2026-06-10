#!/usr/bin/env liquidsoap

# --- telnet server: how the scheduler pushes dynamic content ---
settings.server.telnet := true
settings.server.telnet.bind_addr := "0.0.0.0"
settings.server.telnet.port := 1234

# --- music: the always-on rotating library ---
music = playlist(mode="randomize", reload_mode="watch", "/music")

# --- interrupt queue: news / weather / voiceovers pushed at runtime ---
# registers telnet commands:  interrupt.push <uri>   interrupt.queue   ...
interrupt = request.queue(id="interrupt")

# --- smooth crossfade instead of a hard cut, in both directions ---
# during the ~1s overlap both sources play: music fades down as the voice rises,
# and back up when the interrupt ends.
def crossfade(old, new) =
  add(normalize=false, [
    fade.out(duration=1.0, old),
    fade.in(duration=1.0, new)
  ])
end

# --- priority fallback: anything in the queue wins, else music ---
# track_sensitive=false => an interrupt cuts in without waiting for the song to end
radio = fallback(
  track_sensitive=false,
  transitions=[crossfade, crossfade],
  [interrupt, music]
)

# never go silent (plays blank if every source is empty)
radio = mksafe(radio)

# --- stream to icecast ---
output.icecast(
  %mp3(bitrate=128),
  host="icecast",
  port=8000,
  password="hackme",
  mount="radio.mp3",
  name="RadioPal",
  description="RadioPal test stream",
  radio
)
