from __future__ import annotations

import datetime as dt
import json
import os
import urllib.request
from pathlib import Path

from fastapi import FastAPI, HTTPException
from fastapi.responses import FileResponse
from pydantic import BaseModel

import liquidsoap
from playqueue import QueueItem

WEB = Path(__file__).resolve().parent / "web"
MEDIA = liquidsoap.ROOT / "media"
LANES = ("duck", "takeover", "next")
CLIP_DIRS = ("generated", "station_ids", "short_stories")
AUDIO_SUFFIXES = (".wav", ".mp3", ".m4a", ".m4b")
ICECAST_STATUS = os.environ.get(
    "ICECAST_STATUS_URL",
    f"http://{os.environ.get('ICECAST_HOST', 'localhost')}:8000/status-json.xsl",
)


class PushBody(BaseModel):
    path: str
    lane: str = "takeover"


class ScheduleBody(BaseModel):
    path: str
    lane: str = "takeover"
    play_at: str | None = None
    in_seconds: int | None = None


def _now_playing():
    with urllib.request.urlopen(ICECAST_STATUS, timeout=5) as resp:
        data = json.load(resp)
    source = data.get("icestats", {}).get("source")
    if isinstance(source, list):
        source = source[0] if source else {}
    source = source or {}
    return source.get("title") or source.get("server_name")


def _check_lane(lane):
    if lane not in LANES:
        raise HTTPException(400, f"lane must be one of {LANES}")


def _resolve(path):
    p = Path(path)
    if not p.is_absolute():
        p = liquidsoap.ROOT / p
    p = p.resolve()
    try:
        p.relative_to(liquidsoap.ROOT)
    except ValueError:
        raise HTTPException(400, "path must be inside the app root")
    if not p.exists():
        raise HTTPException(404, f"no such file: {path}")
    return str(p)


def create_app(queue):
    app = FastAPI(title="RadioPal")

    @app.get("/")
    def index():
        return FileResponse(WEB / "index.html")

    @app.get("/api/now-playing")
    def now_playing():
        try:
            return {"title": _now_playing()}
        except Exception:
            return {"title": None}

    @app.get("/api/clips")
    def clips():
        out = []
        for name in CLIP_DIRS:
            base = MEDIA / name
            if not base.exists():
                continue
            for p in sorted(base.rglob("*")):
                if p.suffix.lower() in AUDIO_SUFFIXES:
                    out.append(str(p.relative_to(liquidsoap.ROOT)))
        return out

    @app.get("/api/queue")
    def queue_list():
        return [
            {
                "name": i.name,
                "clip": Path(i.path).name,
                "lane": i.lane,
                "play_at": i.play_at.isoformat(timespec="seconds"),
            }
            for i in queue.snapshot()
        ]

    @app.post("/api/skip")
    def skip():
        return {"reply": liquidsoap.skip()}

    @app.post("/api/push")
    def push(body: PushBody):
        _check_lane(body.lane)
        resolved = _resolve(body.path)
        try:
            reply = liquidsoap.push(resolved, body.lane)
        except Exception as exc:
            raise HTTPException(400, str(exc))
        return {"reply": reply}

    @app.post("/api/schedule")
    def schedule(body: ScheduleBody):
        _check_lane(body.lane)
        resolved = _resolve(body.path)
        if body.play_at:
            when = dt.datetime.fromisoformat(body.play_at)
        elif body.in_seconds is not None:
            when = dt.datetime.now() + dt.timedelta(seconds=body.in_seconds)
        else:
            raise HTTPException(400, "provide play_at or in_seconds")
        queue.add(QueueItem(path=resolved, play_at=when, lane=body.lane, name="manual"))
        return {"play_at": when.isoformat(timespec="seconds")}

    return app
