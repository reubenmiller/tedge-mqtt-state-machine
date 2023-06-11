"""Child updater state machine"""

import logging
from dataclasses import dataclass
from .machine import StateMachine, State, with_transitions, Done
from .context import Context

log = logging.getLogger()


@dataclass
class ChildContext:
    child_id: str
    successful: bool = False
    parent: Context = None


class ChildUpdater(StateMachine):
    def run(self, context: ChildContext = None):
        super().run()
        context = context or ChildContext()
        with_transitions(self, context, init_state=Prepare, delay=0.5)


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
