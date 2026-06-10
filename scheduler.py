from __future__ import annotations

import datetime as dt
import logging

log = logging.getLogger("radiopal.scheduler")


class DailySchedule:
    def __init__(self, at):
        self.at = at

    def next_play_at(self, now, last):
        candidate = now.replace(
            hour=self.at.hour,
            minute=self.at.minute,
            second=0,
            microsecond=0,
        )
        if candidate < now:
            candidate += dt.timedelta(days=1)
        return candidate


class IntervalSchedule:
    def __init__(self, seconds):
        self.delta = dt.timedelta(seconds=seconds)

    def next_play_at(self, now, last):
        if last is None:
            return now + self.delta
        return last + self.delta


class Scheduler:
    def __init__(self, entries, queue, lead_seconds):
        self.entries = entries
        self.queue = queue
        self.lead = dt.timedelta(seconds=lead_seconds)
        self.prepared = {}
        now = dt.datetime.now()
        self.last_play = {e.name: now for e in entries}

    def tick(self, now):
        for entry in self.entries:
            play_at = entry.schedule.next_play_at(now, self.last_play.get(entry.name))
            if play_at - now > self.lead:
                continue
            if self.prepared.get(entry.name) == play_at:
                continue
            try:
                item = entry.action.prepare(play_at)
            except Exception:
                log.exception("prepare failed for %s", entry.name)
                continue
            item.lane = entry.lane
            self.queue.add(item)
            self.prepared[entry.name] = play_at
            self.last_play[entry.name] = play_at
            log.info(
                "prepared %s for %s lane=%s",
                entry.name,
                play_at.isoformat(timespec="seconds"),
                entry.lane,
            )
