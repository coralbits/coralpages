#!/bin/sh

# default variables, can be overridden by environment variables
PORT=${PORT:-8006}
HOST=${HOST:-0.0.0.0}
ENV=${ENV:-production}

./coralpages --listen $HOST:$PORT