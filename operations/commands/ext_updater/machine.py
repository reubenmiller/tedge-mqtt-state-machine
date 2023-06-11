"""State machine"""

import logging
import time
from abc import abstractmethod
from typing import Type
from .context import Context

log = logging.getLogger()


class State:
    @abstractmethod
    def run(self, context: Context):
        raise NotImplementedError


class Done(State):
    def run(self, context: Context) -> State:
        return None


class StateMachine:
    def run(self):
        log.info("Running state machine: %s", self.__class__.__name__)


def with_transitions(
    machine: StateMachine,
    context: Context,
    init_state: Type[State] = None,
    error_state: Type[State] = None,
    delay: float = 0.5,
):
    state = init_state
    while state is not None:
        try:
            log.info(
                "Running state: %s::%s context=%s",
                type(machine).__name__,
                getattr(state, "__name__", type(state).__name__),
                context,
            )
            state = state().run(context)
        except Exception as ex:
            log.error("State error. %s", ex)
            state = error_state

        if delay > 0:
            time.sleep(delay)
