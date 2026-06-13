from __future__ import annotations

from actions.base import Action
from actions.news import NewsAction
from actions.podcast import PodcastAction
from actions.static import StaticAction
from actions.time_check import TimeCheckAction

REGISTRY = {
    cls.type: cls
    for cls in [
        NewsAction,
        PodcastAction,
        StaticAction,
        TimeCheckAction,
    ]
}
