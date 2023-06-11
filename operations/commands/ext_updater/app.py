"""Application"""

import logging
import json
import sys
import time
from typing import List, Type
import threading
from paho.mqtt.client import Client, MQTTMessage
from .machine import StateMachine
from .external_updater import ExternalUpdater, Status
from .context import Context

# Set sensible logging defaults
log = logging.getLogger()
log.setLevel(logging.INFO)
handler = logging.StreamHandler()
handler.setLevel(logging.INFO)
formatter = logging.Formatter("%(asctime)s - %(name)s - %(levelname)s - %(message)s")
handler.setFormatter(formatter)
log.addHandler(handler)


class App:
    def __init__(self, host: str = "localhost", port: int = 1883) -> None:
        client = Client(__name__, clean_session=True)
        self._host = host
        self._port = port
        client.on_connect = self.on_connect
        client.on_message = self.on_message
        self.client = client
        self._workers: List[threading.Thread] = []
    
    def stop(self, message: str):
        state = json.loads(message)

        if state.get("successful", False) == True:
            state["status"] = str(Status.SUCCESSFUL)
        else:
            state["status"] = str(Status.FAILED)
        
        print(json.dumps(state))

    def start(self, message: str):
        state = json.loads(message)
        state["status"] = str(Status.REQUEST)
        print(json.dumps(state))
        

    def run_workflow(self, machine: Type[StateMachine], context: Context):
        log.info("Queuing state machine. %s", machine.__name__)
        worker = threading.Thread(target=machine().run, args=(context,), daemon=True)
        worker.start()
        self._workers.append(worker)

    def wait_all_workflows(self, timeout: float = None):
        for t in self._workers:
            t.join(timeout)

    def on_connect(self, client, userdata, flags, rc):
        log.info("Client is connected. rc=%s", rc)

        if rc == 0:
            self.client.subscribe(
                [
                    ("tedge/operations/+/external/update/+", 2),
                ]
            )

    def on_update(self, client, userdata, msg: MQTTMessage):
        try:
            payload = json.loads(msg.payload.decode("utf8"))
            if payload["status"]:
                pass
        except Exception as ex:
            log.error("Failed to decode message. %s", ex)
        log.info("Received update: %s", msg)

    def on_message(self, client, userdata, msg: MQTTMessage):
        try:
            payload = json.loads(msg.payload.decode("utf8"))
            log.info("Received message: topic=%s, payload=%s", msg.topic, payload)
        except Exception as ex:
            log.error("Unknown message format. %s", ex)
            return

        topic_parts = msg.topic.split("/")
        message_id = topic_parts[-1]
        message_type = "/".join([topic_parts[3], topic_parts[4]])
        log.info("Detected message type: %s", message_type)

        machine = None
        context = None
        if message_type == "external/update":

            status = payload.get("status", "")

            # Only start workflow on trigger status
            if status == str(Status.REQUEST):
                machine = ExternalUpdater
                context = Context(id=message_id, client=self.client, topic=msg.topic)
                context.children = payload.get("children", [])
            else:
                log.info("Ignoring %s workflow status. %s", message_type, status)

        if machine is not None:
            self.run_workflow(machine, context)

    def connect(self):
        self.client.loop_start()
        self.client.connect(self._host, self._port)
        return self

    def wait_forever(self, graceful_shutdown_timeout: float = 10.0):
        try:
            while True:
                time.sleep(1)
        except:
            log.info(
                "Waiting for workflows to finish. timeout=%.1fs",
                graceful_shutdown_timeout,
            )
            self.wait_all_workflows(graceful_shutdown_timeout)
            log.info("Shutting down")
            sys.exit(0)
