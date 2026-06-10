from __future__ import annotations

from actions.base import Action
from actions.time_check import TimeCheckAction

REGISTRY = {
    cls.type: cls
    for cls in [
        TimeCheckAction,
    ]
}
