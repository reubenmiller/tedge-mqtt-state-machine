"""Context"""
from dataclasses import dataclass, field
from typing import List
from paho.mqtt.client import Client


@dataclass
class Context:
    id: str = "123"
    topic: str = ""
    client: Client = None
    successful: bool = False
    reason: str = ""
    children: List[str] = field(default_factory=list)

    def to_dict(self):
        content = {}
        for k, v in self.__dict__.items():
            if k != "client":
                content[k] = v

        return content
