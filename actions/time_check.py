from __future__ import annotations

import tts
from actions.base import Action
from playqueue import QueueItem

DEFAULT_VOICE = "en-US-Chirp3-HD-Charon"


class TimeCheckAction(Action):
    type = "time_check"

    def prepare(self, play_at):
        hour = play_at.strftime("%I").lstrip("0") or "12"
        ampm = play_at.strftime("%p").lower()
        text = f"It's {hour} {ampm}. You're listening to RadioPal."
        voice = self.params.get("voice", DEFAULT_VOICE)
        lang = self.params.get("lang", tts.DEFAULT_LANG)
        path = tts.synthesize(text, voice, lang, tts.GENERATED_DIR / f"{self.name}.mp3")
        return QueueItem(path=str(path), play_at=play_at, name=self.name)
