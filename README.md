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

schedules:
  - name: history_of_philosophy
    lane: next                   # next | duck | takeover
    lead: 300                    # prepare 300s before the slot
    every: "8h"                  # OR  time: "10:00"  (pick one)
    select: sequential           # random | shuffle | sequential
    action:
      type: youtube
      url: "https://www.youtube.com/playlist?list=..."
      cache: content/media/cache
```

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

The scheduler serves a minimal web page (`web/index.html`) with a player,
the currently-playing program, and the schedule table. It listens on
`0.0.0.0:8080` by default.

- `GET /` — the UI.
- `GET /api/state` — JSON: `{ stream_url, now, schedules }`.

The in-page player points at `stream_url` (your Icecast stream). Set it in
`config.yaml`:

```yaml
stream_url: "http://your-host:8003/radio"
```

"Now playing" reflects the most recent clip the scheduler pushed. With the
`next` lane, that clip may be queued behind the current track, so it's a
best-effort indicator rather than exact stream metadata.

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
