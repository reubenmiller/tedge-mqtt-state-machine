"""Child updater state machine"""

import logging
from dataclasses import dataclass
from typing import Dict, Any
from .machine import StateMachine, State, with_transitions, Done
from .context import Context

log = logging.getLogger()


@dataclass
class ChildContext:
    child_id: str
    successful: bool = False
    parent: Context = None

    @property
    def root(self):
        return self.parent

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> "ChildContext":
        context = cls()
        for k, v in d.items():
            if hasattr(context, k):
                setattr(context, k, v)

        return context

    def to_dict(self):
        content = {}
        for k, v in self.__dict__.items():
            if k != "parent":
                content[k] = v

        return content


class Prepare(State):
    def run(self, context: ChildContext):
        return Install


class Install(State):
    def run(self, context: ChildContext):
        return Verify


class Verify(State):
    def run(self, context: ChildContext):
        if "2" in context.child_id:
            # Simulate an error
            return Rollback
        return Commit


class Commit(State):
    def run(self, context: ChildContext):
        context.successful = True
        return Finalize


class Rollback(State):
    def run(self, context: ChildContext):
        return Finalize


class Finalize(State):
    def run(self, context: ChildContext):
        return Done


class ChildUpdater(StateMachine):
    def run(self, context: ChildContext = None, init_state=None):
        init_state = init_state or Prepare
        super().run(context, init_state=init_state)
        context = context or ChildContext()
        with_transitions(self, context, init_state=Prepare, delay=1)
