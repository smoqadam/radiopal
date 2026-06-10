from __future__ import annotations

import json
import os
import urllib.parse
import urllib.request

import llm
import tts
from actions.base import Action
from playqueue import QueueItem

ENDPOINT = "https://newsapi.org/v2/top-headlines"

SYSTEM = (
    "You are the news anchor for RadioPal, a 24/7 radio station. "
    "Using the date, time, and headlines provided, deliver a short spoken news "
    "bulletin for radio: greet the listeners and mention the day and time, then "
    "cover the headlines in a few natural sentences, then a brief sign-off. "
    "Speak in English. Do not use emojis, markdown, quotation marks, or stage "
    "directions. Output only the words to be spoken aloud."
)


def _fetch(country, category, page_size, api_key):
    query = urllib.parse.urlencode({
        "country": country,
        "category": category,
        "pageSize": page_size,
        "apiKey": api_key,
    })
    with urllib.request.urlopen(f"{ENDPOINT}?{query}", timeout=10) as resp:
        data = json.load(resp)
    if data.get("status") != "ok":
        raise RuntimeError(f"newsapi error: {data.get('code')} {data.get('message')}")
    items = []
    for article in data.get("articles", []):
        title = article.get("title")
        if not title:
            continue
        description = article.get("description") or ""
        items.append((title, description))
    return items


class NewsAction(Action):
    type = "news"

    def prepare(self, play_at):
        api_key = os.environ["NEWSAPI_KEY"]
        country = self.params.get("country", "us")
        category = self.params.get("category", "general")
        count = self.params.get("count", 5)
        voice = self.params.get("voice", "en-US-Chirp3-HD-Charon")
        lang = self.params.get("lang", tts.DEFAULT_LANG)

        headlines = _fetch(country, category, count, api_key)
        if not headlines:
            raise RuntimeError(f"no headlines for {country}/{category}")

        when = play_at.strftime("%A, %B %-d, %-I:%M %p")
        lines = [f"- {t}: {d}" if d else f"- {t}" for t, d in headlines]
        user = f"Date and time: {when}.\nCategory: {category}.\nTop headlines:\n" + "\n".join(lines)

        text = llm.generate(SYSTEM, user, max_tokens=self.params.get("max_tokens", 600))
        path = tts.synthesize(text, voice, lang, tts.GENERATED_DIR / f"{self.name}.mp3")
        return QueueItem(path=str(path), play_at=play_at, name=self.name)
