# RadioPal

RadioPal is a self-hosted automation tool for 24/7 internet radio. It manages a continuous music stream and schedules TTS-generated spoken content, such as news bulletins, short stories, and station IDs to play at defined intervals. It functions as an orchestration layer over standard radio software like __Icecast__ and __Liquidsoap__.


## What it is

RadioPal is three cooperating pieces:

- **Icecast** -- the streaming server. This is the URL listeners connect to.
- **Liquidsoap** -- the audio engine. It plays the music playlist and mixes in
  spoken segments, then sends the result to Icecast.
- **Scheduler** -- a small Python daemon (this repo). It decides what to play and
  when, generates dynamic audio (LLM & TTS), and hands finished clips to
  Liquidsoap at the right moment.

The music stream never stops. Spoken content arrives in one of two ways:

- **duck** -- the music dips in volume and the voice plays over it, then the
  music comes back up. Good for short station IDs.
- **takeover** -- the music fades out, the segment plays in full, then the music
  fades back in. Good for news and stories.

## Why it exists


I saw [this](https://old.reddit.com/r/digitalminimalism/comments/1tes8yu/i_replaced_spotify_with_a_homemade_fm_radio/) post on reddit the other day and liked the idea of having a custom radio station that plays the things I actually want to listen to: my music library, my audiobooks, custom news bulletins, or basically any audio content I own or can generate via an LLM and TTS.

I have a few ideas in mind for the future:

  - Connecting my calendar so the "host" can remind me of my appointments.

  - Listening to my audiobook library every night at 9 PM.

  - Automatically playing jazz when it's raining outside.
  top RSS feed headlines.

  - Scraping my favorite unread email newsletters and turning them into a 10-minute podcast segment during my lunch break.

  - Tying into smart home events, so the host casually announces when the laundry is done or the 3D printer finishes a job.


## How it works

```
                 generates audio              telnet push
  Scheduler  ----------------------------->  Liquidsoap  ---->  Icecast  ---->  listeners
  (Python)     TTS / LLM / static clips       (audio engine)    (stream)        (browser, etc.)
```


## Requirements

- Docker and Docker Compose
- API credentials, depending on which actions you enable:
  - Google Cloud Text-to-Speech service-account JSON (for spoken generated content).
  - A LiteLLM-compatible LLM key, e.g. DeepSeek (for news/story text).
  - A newsapi.org key (for the news action).

## Setup

1. **Configuration file.** Copy the example and fill it in:

   ```
   cp .env.example .env

  ##
   MUSIC_DIR=./music
   STATION_IDS_DIR=./station_ids
   GENERATED_DIR=./generated
   SHORT_STORIES_DIR=./short_stories

   DEEPSEEK_API_KEY=...
   NEWSAPI_KEY=...
   ```

2. **Google credentials.** Place your Google service-account JSON at
   `creds/radiopal-tts.json`.

3. **Music.** Put mp3 files in your `MUSIC_DIR`. Liquidsoap watches the folder
   and picks them up automatically.

4. **Schedule.** Edit `config.yaml` (see below).

## Run

Build and start everything:

```
docker compose up -d --build
```

Watch the scheduler:

```
docker compose logs -f scheduler
```

Listen to the stream:

```
http://localhost:8000/radio.mp3
```

To stop:

```
docker compose down
```

## Configuration

`config.yaml` has two global settings and a list of actions:

```yaml
lead_seconds: 300     # how far ahead to prepare a segment
tick_seconds: 20      # how often the scheduler wakes up

actions:
  - name: station_id
    action: station_id
    lane: duck
    every: "15m"

  - name: news
    action: news
    lane: takeover
    time: "19:45"
    params:
      country: us
      category: general
      count: 5

  - name: sport
    action: news
    lane: takeover
    time: "19:50"
    params:
      country: us
      category: sports
      count: 5

```

