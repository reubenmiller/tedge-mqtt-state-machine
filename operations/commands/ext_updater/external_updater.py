"""External updater state machine"""

import logging
import json
from enum import Enum
from .machine import StateMachine, State, with_transitions, Done
from .context import Context
from .child_updater import ChildUpdater, ChildContext

log = logging.getLogger()


class Status(Enum):
    FAILED = "failed"
    SUCCESSFUL = "successful"
    REQUEST = "external_request"
    RESPONSE = "external_response"

    def __str__(self) -> str:
        return str(self.value)


class ExternalUpdater(StateMachine):
    def run(self, context: Context):
        super().run()
        with_transitions(self, context, init_state=ExternalStart)


class ExternalStart(State):
    def run(self, context: Context) -> State:
        log.info("Starting to update child devices")

        # Update child devices
        all_children_successful = True
        child_errors = []
        for child in context.children:
            child_context = ChildContext(child_id=child, parent=context)
            ChildUpdater().run(child_context)
            if not child_context.successful:
                all_children_successful = False
                child_errors.append(child)
        
        context.successful = all_children_successful

        if not all_children_successful:
            log.info("Rolling back version of children")
            context.reason = f"Some child devices failed to update. {child_errors}"
            return Reboot

        return Finalize


class Rollback(State):
    def run(self, context: Context) -> State:
        log.info("Rolling back main device. %s", context)
        return Reboot


class Reboot(State):
    def run(self, context: Context) -> State:
        log.info("Restarting. %s", context)
        return Finalize


class Finalize(State):
    def run(self, context: Context) -> State:
        log.info("Finished updating child devices. %s", context)
        
        if context.client:
            payload = context.to_dict()
            payload["status"] = str(Status.RESPONSE)
            serial_payload = json.dumps(payload)
            log.info("Publishing message: topic=%s, payload=%s", context.topic, payload)
            if context.topic:
                context.client.publish(context.topic, serial_payload, qos=1, retain=True)
        return Done
