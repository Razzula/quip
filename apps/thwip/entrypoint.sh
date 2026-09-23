#!/bin/sh
set -eu

/usr/local/bin/thwip &
THWIP_PID=$!

nginx -g 'daemon off;' &
NGINX_PID=$!

cleanup() {
    kill "$THWIP_PID" "$NGINX_PID" 2>/dev/null || true
    wait "$THWIP_PID" "$NGINX_PID" 2>/dev/null || true
}

trap cleanup INT TERM EXIT

while :; do
    if ! kill -0 "$THWIP_PID" 2>/dev/null; then
        wait "$THWIP_PID" || true
        exit 1
    fi

    if ! kill -0 "$NGINX_PID" 2>/dev/null; then
        wait "$NGINX_PID" || true
        exit 1
    fi

    sleep 1
done
