settings.server.telnet := true
settings.server.telnet.bind_addr := "0.0.0.0"
settings.server.telnet.port := 1234
settings.log.level := 3

music = playlist(id="music", mode="randomize", reload_mode="watch", "music")
music = mksafe(music)

next_q     = request.queue(id="next")
duck_q     = request.queue(id="duck")
takeover_q = request.queue(id="takeover")

base = fallback(id="base", track_sensitive=true, [next_q, music])

ducked = amplify(id="duck_bed", {if source.is_ready(duck_q) then 0.25 else 1.0 end}, base)
base = add(id="duck_mix", normalize=false, [ducked, duck_q])

radio = fallback(id="radio", track_sensitive=false, [takeover_q, base])
radio = mksafe(radio)

output.icecast(
  %mp3(bitrate=128),
  host="icecast",
  port=8000,
  password="hackme",
  mount="radio",
  name="RadioPal",
  description="RadioPal stream",
  radio
)
