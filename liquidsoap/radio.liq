settings.server.telnet := true
settings.server.telnet.bind_addr := "0.0.0.0"
settings.server.telnet.port := 1234

music = playlist(mode="randomize", reload_mode="watch", "/music")

duck = request.queue(id="duck")
bed = smooth_add(p=0.2, duration=1.0, normal=music, special=duck)

takeover = request.queue(id="takeover")

def crossfade(old, new) =
  add(normalize=false, [
    fade.out(duration=1.0, old),
    fade.in(duration=1.0, new)
  ])
end

radio = fallback(track_sensitive=false, transitions=[crossfade, crossfade], [takeover, bed])
radio = mksafe(radio)

output.icecast(
  %mp3(bitrate=128),
  host="icecast",
  port=8000,
  user=environment.get(default="source", "ICECAST_SOURCE_USER"),
  password=environment.get("ICECAST_SOURCE_PASSWORD"),
  mount="radio.mp3",
  name="RadioPal",
  description="RadioPal test stream",
  radio
)
