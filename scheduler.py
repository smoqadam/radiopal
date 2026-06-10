from __future__ import annotations

import datetime as dt
import logging

log = logging.getLogger("radiopal.scheduler")


class Scheduler:
    def __init__(self, entries, queue, lead_seconds):
        self.entries = entries
        self.queue = queue
        self.lead = dt.timedelta(seconds=lead_seconds)
        self.prepared = {}

    def _next_play_at(self, play_time, now):
        candidate = now.replace(
            hour=play_time.hour,
            minute=play_time.minute,
            second=0,
            microsecond=0,
        )
        if candidate < now:
            candidate += dt.timedelta(days=1)
        return candidate

    def tick(self, now):
        for entry in self.entries:
            play_at = self._next_play_at(entry.play_time, now)
            if play_at - now > self.lead:
                continue
            if self.prepared.get(entry.name) == play_at:
                continue
            try:
                item = entry.action.prepare(play_at)
            except Exception:
                log.exception("prepare failed for %s", entry.name)
                continue
            self.queue.add(item)
            self.prepared[entry.name] = play_at
            log.info("prepared %s for %s", entry.name, play_at.isoformat(timespec="seconds"))
