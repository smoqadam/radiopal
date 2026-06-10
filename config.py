from __future__ import annotations

import datetime as dt
from dataclasses import dataclass
from pathlib import Path

import yaml

from actions import REGISTRY

CONFIG_PATH = Path(__file__).resolve().parent / "config.yaml"


@dataclass
class Entry:
    name: str
    play_time: dt.time
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
        action = cls(raw["name"], raw.get("params", {}))
        entries.append(Entry(raw["name"], _parse_time(raw["time"]), action))

    return Config(lead_seconds, tick_seconds, entries)
