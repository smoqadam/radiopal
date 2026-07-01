# RadioPal

A self-hosted internet radio station. A small Rust scheduler decides *what* plays
and *when*, prepares the audio ahead of time, and feeds it to
[Liquidsoap](https://www.liquidsoap.info/), which streams a continuous MP3 to
[Icecast](https://icecast.org/).

```
scheduler (Rust) ──push file path──> liquidsoap ──stream──> icecast ──> listeners
   │  reads config.yaml                  │
   │  downloads / picks clips            └─ mixes a music bed under the programming
   └─ pushes over telnet (port 1234)
```

## How it works

The scheduler ticks every few seconds. For each schedule it:

1. Fires when the slot is near (`lead` seconds before the slot time).
2. Resolves the **action** into a local audio file (downloading if needed).
3. Pushes the file to a Liquidsoap **lane** over telnet.

Between programmed clips, Liquidsoap plays a shuffled **music bed** from the
`music/` playlist, so the stream is never silent.

## Configuration

Everything lives in `config/config.yaml`.

```yaml
tick_seconds: 10                 # scheduler poll interval (optional)
liquidsoap_addr: "127.0.0.1:1234"
station_name: "Radio Saeed"      # radio's display name (web ui)
stream_url: "http://host:8003/radio"   # where the web player connects

schedules:
  - name: history_of_philosophy
    title: "History of Philosophy" # human-facing label (web ui + on-air metadata)
    lane: next                   # next | duck | takeover
    lead: 300                    # prepare 300s before the slot
    every: "8h"                  # OR  time: "10:00"  (pick one)
    select: sequential           # random | shuffle | sequential
    action:
      type: youtube
      url: "https://www.youtube.com/playlist?list=..."
      cache: content/media/cache
```

`title` is optional and falls back to `name`. It's what listeners see in the web
UI and what Icecast broadcasts as the track title.

### Timing — `every` vs `time`

Each schedule uses exactly one:

- `time: "10:00"` — daily at that clock time.
- `every: "8h"` — repeating, aligned to midnight (`8h` → 00:00, 08:00, 16:00).
  Units: `h`, `m`, `s`.

### Lanes

| Lane       | Behaviour                                                    |
|------------|-------------------------------------------------------------|
| `next`     | Queued; plays after the current track. Normal programming.  |
| `duck`     | Overlaid on top; the bed/programming ducks to 25%.          |
| `takeover` | Preempts everything and plays immediately.                  |

### Selectors

- `random` — independent random pick each time.
- `shuffle` — random without repeats until the pool is exhausted.
- `sequential` — plays in order, resuming across restarts.

Selector positions are persisted to `RADIOPAL_STATE_FILE` so `sequential`/`shuffle`
survive restarts.

## Actions

**`youtube`** — lists a channel/playlist with `yt-dlp`, picks one via the selector,
and downloads audio as MP3 into `cache`.

```yaml
action:
  type: youtube
  url: "https://www.youtube.com/@channel"
  cache: content/media/cache
```

**`static`** — plays local audio files from a directory (searched recursively).

```yaml
action:
  type: static
  dir: content/radio/short_stories
```

**`ganjoor`** — fetches a random Persian poem and its recitation from
[ganjoor.net](https://ganjoor.net) and plays the audio. If the poem has no
recitation, the slot simply plays nothing.

```yaml
action:
  type: ganjoor
  poet_id: 2                     # 0 = random poet (e.g. حافظ 2, سعدی 7, خیام 3)
  cache: content/media/cache
```

## Web UI

The scheduler serves a minimal web page (`web/index.html`) with a player, the
station name, the currently-playing program, and a schedule table with live
countdowns to each program's next run (the soonest is flagged "up next"). It
listens on `0.0.0.0:8080` by default.

- `GET /` — the UI.
- `GET /api/state` — JSON: `{ station, stream_url, now, schedules }`, where each
  schedule includes a `next_run` (unix seconds) for the countdown.

The in-page player points at `stream_url` (your Icecast stream); set it and
`station_name` in `config.yaml`.

"Now playing" is read live from Liquidsoap (`request.on_air`), so it reflects the
actual on-air audio — music bed or program. If Liquidsoap can't be reached, it
falls back to the last clip the scheduler pushed.

## Running

With Docker Compose (scheduler + Liquidsoap + Icecast):

```bash
cp .env.example .env     # set host paths / image
docker compose up -d --build
```

The stream is served by Icecast (default mount `/radio`).

### Environment variables

| Variable                   | Default                  | Purpose                          |
|----------------------------|--------------------------|----------------------------------|
| `RADIOPAL_CONFIG`          | `config/config.yaml`     | Path to the config file          |
| `RADIOPAL_LIQUIDSOAP_ADDR` | `127.0.0.1:1234`         | Liquidsoap telnet address        |
| `RADIOPAL_STATE_FILE`      | `selector_state.json`    | Where selector state is persisted |
| `RADIOPAL_WEB_ADDR`        | `0.0.0.0:8080`           | Web UI bind address              |

## Development

```bash
cargo build
cargo test
cargo run            # loads config/config.yaml, needs Liquidsoap on :1234
```

Requires `yt-dlp` (with `deno`), `ffmpeg`, and `curl` on the host for the
`youtube` and `ganjoor` actions — all bundled in the Docker image.
