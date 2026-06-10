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
                reply = liquidsoap.push(item.path)
            except Exception:
                log.exception("play failed for %s", item.name)
                continue
            log.info("played %s -> %r", item.name, reply)
