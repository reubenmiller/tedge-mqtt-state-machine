"""State machine"""

import logging
import time
import json
from abc import abstractmethod
from typing import Type, Any
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
    def run(self, context: Context, init_state: Any = None):
        log.info("Running state machine: %s", self.__class__.__name__)


def with_transitions(
    machine: StateMachine,
    context: Context,
    init_state: Type[State] = None,
    error_state: Type[State] = None,
    delay: float = 1.0,
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
            prev_state = state
            state = state().run(context)
            publish(context, machine, state, prev_state)
        except Exception as ex:
            log.error("State error. %s", ex, exc_info=True)
            state = error_state

        if delay > 0:
            time.sleep(delay)


def name(state) -> str:
    if state is None:
        return ""

    return getattr(state, "__name__")


def publish(
    context: Context, machine, state: Type[State], prev_state: Type[State] = None
):
    root = context.root
    payload = root.to_dict()
    payload["status"] = state.__name__ if state else ""

    root.client.publish(
        "tedge/events/tedge_StateMachineTransition",
        json.dumps(
            {
                "text": f"[{type(machine).__name__}] state machine: [{name(prev_state)}] ➜ [{name(state)}]",
                "context": payload,
            }
        ),
    )

    msg = root.client.publish(root.topic, json.dumps(payload), qos=1, retain=True)
    wait_for_publish(msg, 10)


def wait_for_publish(msg, timeout: float = None) -> bool:
    if not timeout:
        msg.wait_for_publish()
        return

    limit = time.monotonic() + timeout
    did_timeout = False
    while not msg.is_published():
        if time.monotonic() < limit:
            did_timeout = True
            break
        time.sleep(0.05)

    return did_timeout
