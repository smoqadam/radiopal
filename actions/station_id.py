from __future__ import annotations

import pathlib
import random

from actions.base import Action
from playqueue import QueueItem

DIR = pathlib.Path(__file__).resolve().parent.parent / "station_ids"


def _daypart(hour):
    if 5 <= hour < 11:
        return "morning"
    if 11 <= hour < 17:
        return "day"
    if 17 <= hour < 22:
        return "evening"
    return "night"


# reads station IDs as duck (with bg music)
class StationIdAction(Action):
    type = "station_id"

    def prepare(self, play_at):
        pool = DIR / _daypart(play_at.hour)
        files = sorted(p for p in pool.glob("*") if p.suffix.lower() in (".wav", ".mp3"))
        if not files:
            raise FileNotFoundError(f"no station IDs in {pool}")
        chosen = random.choice(files)
        return QueueItem(path=str(chosen), play_at=play_at, name=self.name)
