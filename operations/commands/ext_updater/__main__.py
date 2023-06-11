"""main"""
from .app import App

import sys

if len(sys.argv) > 1:
    print(f"Running cli: {sys.argv}", file=sys.stderr)
    command = sys.argv[1]
    state = "{}"
    if len(sys.argv) > 2:
        state = sys.argv[2]

    if command == "start":
        App().start(state)
    elif command == "stop":
        App().stop(state)
    else:
        sys.exit(1)
else:
    App().connect().wait_forever()
