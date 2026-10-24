#!/usr/bin/env bash
# Restarts the dev server whenever the binary is rebuilt, starting the new
# server before stopping the old one. Run `cargo watch -x build` alongside it.
set -uo pipefail
cd "$(dirname "$0")"

BIN=target/debug/sigfrids-world
PORT=7878

set -a
. ./.env
set +a

mtime() {
    stat -c %Y "$BIN" 2>/dev/null || stat -f %m "$BIN" 2>/dev/null
}

old=""
last=""

trap '[ -n "$old" ] && kill "$old" 2>/dev/null; wait; exit 0' INT TERM

while true; do
    current=$(mtime)
    if [ -n "$current" ] && [ "$current" != "$last" ]; then
        last=$current

        "$BIN" &
        new=$!

        # Wait until the new server is listening, or give up if it exits.
        until lsof -a -p "$new" -iTCP:"$PORT" -sTCP:LISTEN >/dev/null 2>&1; do
            if ! kill -0 "$new" 2>/dev/null; then
                echo "dev.sh: new server exited, keeping the old one" >&2
                new=""
                break
            fi
            sleep 0.1
        done

        if [ -n "$new" ]; then
            if [ -n "$old" ]; then
                kill "$old" 2>/dev/null
                wait "$old" 2>/dev/null
            fi
            old=$new
            echo "dev.sh: now serving from pid $old"
        fi
    fi
    sleep 0.5
done
