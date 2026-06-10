from __future__ import annotations

from actions.base import Action
from actions.station_id import StationIdAction
from actions.time_check import TimeCheckAction

REGISTRY = {
    cls.type: cls
    for cls in [
        StationIdAction,
        TimeCheckAction,
    ]
}
