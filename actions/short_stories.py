from __future__ import annotations

import json
import pathlib
import random

from actions.base import Action
from playqueue import QueueItem

DIR = pathlib.Path(__file__).resolve().parent.parent / "short_stories"
STATE = DIR / ".played.json"


def _load_played():
    if STATE.exists():
        return set(json.loads(STATE.read_text()))
    return set()


def _save_played(played):
    STATE.write_text(json.dumps(sorted(played)))


# reads short stories as takeover
class ShortStoriesAction(Action):
    type = "short_stories"

    def prepare(self, play_at):
        files = sorted(p for p in DIR.glob("*") if p.suffix.lower() in (".wav", ".mp3"))
        if not files:
            raise FileNotFoundError(f"no short stories in {DIR}")
        played = _load_played()
        remaining = [p for p in files if p.name not in played]
        if not remaining:
            played = set()
            remaining = files
        chosen = random.choice(remaining)
        played.add(chosen.name)
        _save_played(played)
        return QueueItem(path=str(chosen), play_at=play_at, name=self.name)
