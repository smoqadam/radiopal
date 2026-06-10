from __future__ import annotations

import pathlib
import random

from actions.base import Action
from playqueue import QueueItem

DIR = pathlib.Path(__file__).resolve().parent.parent / "station_ids"

# reads station IDs as duck (with bg music)
class StationIdAction(Action):
    type = "station_id"

    def prepare(self, play_at):
        files = sorted(DIR.glob("*.mp3"))
        if not files:
            raise FileNotFoundError(f"no station IDs in {DIR}")
        chosen = random.choice(files)
        return QueueItem(path=str(chosen), play_at=play_at, name=self.name)
