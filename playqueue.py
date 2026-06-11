from __future__ import annotations

import threading
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
        self._lock = threading.Lock()

    def add(self, item):
        with self._lock:
            self._items.append(item)

    def due(self, now):
        with self._lock:
            ready = sorted(
                (i for i in self._items if i.play_at <= now),
                key=lambda i: i.play_at,
            )
            for item in ready:
                self._items.remove(item)
            return ready

    def snapshot(self):
        with self._lock:
            return sorted(self._items, key=lambda i: i.play_at)

    def __len__(self):
        with self._lock:
            return len(self._items)
