#!/bin/sh

# default variables, can be overridden by environment variables
PORT=${PORT:-8004}
HOST=${HOST:-0.0.0.0}
RELOAD=${RELOAD:-0}
ENV=${ENV:-production}
DEFAULT_TIMEZONE=${DEFAULT_TIMEZONE:-UTC}
LOG_LEVEL=${LOG_LEVEL:-INFO}

# parse options
if [ "$ENV" = "devel" ]; then
    RELOAD=1
fi

if [ "$RELOAD" = "1" ]; then
    RELOAD_ARG="--reload"
fi

# run the server
exec uv run uvicorn main:app --host $HOST --port $PORT $RELOAD_ARG
