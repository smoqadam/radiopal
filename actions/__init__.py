from __future__ import annotations

from actions.base import Action
from actions.news import NewsAction
from actions.station_id import StationIdAction
from actions.time_check import TimeCheckAction
from actions.short_stories import ShortStoriesAction

REGISTRY = {
    cls.type: cls
    for cls in [
        NewsAction,
        StationIdAction,
        TimeCheckAction,
        ShortStoriesAction,
    ]
}
