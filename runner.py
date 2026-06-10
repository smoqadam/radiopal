from __future__ import annotations

import logging

import liquidsoap

log = logging.getLogger("radiopal.runner")


class Runner:
    def __init__(self, queue):
        self.queue = queue

    def tick(self, now):
        for item in self.queue.due(now):
            try:
                reply = liquidsoap.push(item.path, item.lane)
            except Exception:
                log.exception("play failed for %s", item.name)
                continue
            log.info("played %s lane=%s -> %r", item.name, item.lane, reply)
