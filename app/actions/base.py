from __future__ import annotations

from abc import ABC, abstractmethod


class Action(ABC):
    type = "action"

    def __init__(self, name, params):
        self.name = name
        self.params = params

    @abstractmethod
    def prepare(self, play_at):
        ...
