"""Context"""
from dataclasses import dataclass, field
from typing import List, Dict, Any
from paho.mqtt.client import Client


@dataclass
class Context:
    id: str = "123"
    topic: str = ""
    client: Client = None
    successful: bool = False
    reason: str = ""
    children: List[str] = field(default_factory=list)

    @property
    def root(self):
        return self

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> "Context":
        context = cls()
        for k, v in d.items():
            if hasattr(context, k):
                setattr(context, k, v)

        return context

    def to_dict(self):
        content = {}
        for k, v in self.__dict__.items():
            if k != "client":
                content[k] = v

        return content
