from __future__ import annotations

import argparse
import datetime as dt
import logging
import os
import sys
import threading

import uvicorn

import config as config_mod
from api import create_app
from playqueue import PlayQueue
from runner import Runner
from scheduler import Scheduler


def _loop(scheduler, runner, tick_seconds, stop):
    while not stop.is_set():
        now = dt.datetime.now()
        scheduler.tick(now)
        runner.tick(now)
        stop.wait(tick_seconds)


def main(argv):
    p = argparse.ArgumentParser(prog="radiopal")
    p.add_argument("--config", default=None)
    args = p.parse_args(argv)

    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s %(levelname)s %(name)s: %(message)s",
    )
    log = logging.getLogger("radiopal")

    cfg = config_mod.load(args.config)
    queue = PlayQueue()
    scheduler = Scheduler(cfg.entries, queue, cfg.lead_seconds)
    runner = Runner(queue)

    log.info(
        "up; %d entry(ies), lead=%ds, tick=%ds",
        len(cfg.entries),
        cfg.lead_seconds,
        cfg.tick_seconds,
    )

    stop = threading.Event()
    worker = threading.Thread(
        target=_loop,
        args=(scheduler, runner, cfg.tick_seconds, stop),
        daemon=True,
    )
    worker.start()

    port = int(os.environ.get("RADIOPAL_API_PORT", "8080"))
    try:
        uvicorn.run(create_app(queue), host="0.0.0.0", port=port, log_level="warning")
    finally:
        stop.set()


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
