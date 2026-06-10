from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime


@dataclass
class QueueItem:
    path: str
    play_at: datetime
    name: str
    lane: str = "takeover"
    metadata: dict = field(default_factory=dict)


class PlayQueue:
    def __init__(self):
        self._items = []

    def add(self, item):
        self._items.append(item)

    def due(self, now):
        ready = sorted(
            (i for i in self._items if i.play_at <= now),
            key=lambda i: i.play_at,
        )
        for item in ready:
            self._items.remove(item)
        return ready

    def __len__(self):
        return len(self._items)
