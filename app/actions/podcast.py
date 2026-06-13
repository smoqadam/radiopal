from __future__ import annotations

import hashlib
import pathlib
import random
import re
import urllib.request
import xml.etree.ElementTree as ET

import liquidsoap
from actions.base import Action
from playqueue import QueueItem

ROOT = pathlib.Path(__file__).resolve().parent.parent / "media"
UA = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36"


def _slug(value):
    return re.sub(r"[^A-Za-z0-9._-]+", "_", value).strip("_")[:120]


class PodcastAction(Action):
    type = "podcast"

    def prepare(self, play_at):
        episodes = self._episodes(self.params["feed"])
        if not episodes:
            raise FileNotFoundError(f"no episodes in {self.params['feed']}")
        guid, url = random.choice(episodes)
        dest = self._dest(guid, url)
        if not (dest.exists() and dest.stat().st_size > 0):
            self._download(url, dest)
        return QueueItem(path=str(dest), play_at=play_at, name=self.name)

    def _dir(self):
        d = ROOT / "podcasts" / self.name
        d.mkdir(parents=True, exist_ok=True)
        return d

    def _dest(self, guid, url):
        digest = hashlib.sha1(guid.encode()).hexdigest()[:8]
        base = _slug(urllib.request.url2pathname(url.split("?", 1)[0]).rsplit("/", 1)[-1])
        if not base.lower().endswith(liquidsoap.AUDIO_SUFFIXES):
            base = f"{base}.mp3" if base else "episode.mp3"
        return self._dir() / f"{digest}-{base}"

    def _episodes(self, feed):
        req = urllib.request.Request(feed, headers={"User-Agent": UA})
        with urllib.request.urlopen(req, timeout=30) as resp:
            tree = ET.parse(resp)
        out = []
        for item in tree.iter("item"):
            enclosure = item.find("enclosure")
            if enclosure is None:
                continue
            url = enclosure.get("url")
            if not url:
                continue
            guid = item.findtext("guid") or url
            out.append((guid.strip(), url.strip()))
        return out

    def _download(self, url, dest):
        req = urllib.request.Request(url, headers={"User-Agent": UA})
        tmp = dest.with_suffix(dest.suffix + ".part")
        with urllib.request.urlopen(req, timeout=60) as resp, open(tmp, "wb") as out:
            while True:
                chunk = resp.read(1 << 16)
                if not chunk:
                    break
                out.write(chunk)
        tmp.replace(dest)
