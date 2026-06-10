from __future__ import annotations

import datetime as dt
from dataclasses import dataclass
from pathlib import Path

import yaml

from actions import REGISTRY
from scheduler import DailySchedule, IntervalSchedule

CONFIG_PATH = Path(__file__).resolve().parent / "config.yaml"
LANES = ("duck", "takeover")


@dataclass
class Entry:
    name: str
    schedule: object
    lane: str
    action: object


@dataclass
class Config:
    lead_seconds: int
    tick_seconds: int
    entries: list


def _parse_time(value):
    if not isinstance(value, str):
        raise ValueError(f'time must be a quoted "HH:MM" string, got {value!r}')
    hh, mm = value.split(":")
    return dt.time(int(hh), int(mm))


def _parse_every(value):
    if not isinstance(value, str) or len(value) < 2:
        raise ValueError(f'every must be a string like "15m", got {value!r}')
    unit = value[-1]
    factor = {"s": 1, "m": 60, "h": 3600}.get(unit)
    if factor is None:
        raise ValueError(f'every must end in s/m/h, got {value!r}')
    return int(value[:-1]) * factor


def _build_schedule(raw):
    has_time = "time" in raw
    has_every = "every" in raw
    if has_time == has_every:
        raise ValueError(f"entry {raw.get('name')!r} needs exactly one of 'time' or 'every'")
    if has_time:
        return DailySchedule(_parse_time(raw["time"]))
    return IntervalSchedule(_parse_every(raw["every"]))


def load(path=None):
    path = Path(path) if path else CONFIG_PATH
    data = yaml.safe_load(path.read_text())

    lead_seconds = int(data.get("lead_seconds", 300))
    tick_seconds = int(data.get("tick_seconds", 20))

    entries = []
    for raw in data.get("actions", []):
        atype = raw["action"]
        cls = REGISTRY.get(atype)
        if cls is None:
            raise ValueError(f"unknown action {atype!r}; known: {sorted(REGISTRY)}")
        lane = raw.get("lane", "takeover")
        if lane not in LANES:
            raise ValueError(f"lane must be one of {LANES}, got {lane!r}")
        action = cls(raw["name"], raw.get("params", {}))
        entries.append(Entry(raw["name"], _build_schedule(raw), lane, action))

    return Config(lead_seconds, tick_seconds, entries)
