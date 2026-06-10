from __future__ import annotations

import argparse
import datetime as dt
import logging
import sys
import time

import config as config_mod
from playqueue import PlayQueue
from runner import Runner
from scheduler import Scheduler


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
    while True:
        now = dt.datetime.now()
        scheduler.tick(now)
        runner.tick(now)
        time.sleep(cfg.tick_seconds)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
