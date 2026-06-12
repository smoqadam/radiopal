from __future__ import annotations

import json
import pathlib
import random

from actions.base import Action
from playqueue import QueueItem

ROOT = pathlib.Path(__file__).resolve().parent.parent / "media"
AUDIO = (".wav", ".mp3", ".m4a", ".m4b")


def _daypart(hour):
    if 5 <= hour < 11:
        return "morning"
    if 11 <= hour < 17:
        return "day"
    if 17 <= hour < 22:
        return "evening"
    return "night"


class StaticAction(Action):
    type = "static"

    def prepare(self, play_at):
        root = ROOT / self.params["dir"]
        pool = root / _daypart(play_at.hour) if self.params.get("daypart") else root
        files = sorted(p for p in pool.glob("*") if p.suffix.lower() in AUDIO)
        if not files:
            raise FileNotFoundError(f"no audio in {pool}")
        select = self.params.get("select", "random")
        if select == "random":
            chosen = random.choice(files)
        elif select == "shuffle":
            chosen = self._shuffle(root, files)
        elif select == "sequential":
            chosen = self._sequential(root, files)
        else:
            raise ValueError(f"unknown select {select!r}")
        return QueueItem(path=str(chosen), play_at=play_at, name=self.name)

    def _state(self, root):
        return root / f".{self.name}.json"

    def _shuffle(self, root, files):
        state = self._state(root)
        played = set()
        if state.exists():
            data = json.loads(state.read_text())
            if isinstance(data, list):
                played = set(data)
        remaining = [p for p in files if p.name not in played]
        if not remaining:
            played = set()
            remaining = files
        chosen = random.choice(remaining)
        played.add(chosen.name)
        state.write_text(json.dumps(sorted(played)))
        return chosen

    def _sequential(self, root, files):
        state = self._state(root)
        last = None
        if state.exists():
            data = json.loads(state.read_text())
            if isinstance(data, dict):
                last = data.get("last")
        names = [p.name for p in files]
        idx = (names.index(last) + 1) % len(files) if last in names else 0
        chosen = files[idx]
        state.write_text(json.dumps({"last": chosen.name}))
        return chosen
