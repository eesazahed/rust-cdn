#!/bin/bash

cd /Users/path || exit 1
export $(grep -E '^PORT=' .env | xargs)
PORT=${PORT:-12345}
kill -9 $(lsof -ti ":$PORT") 2>/dev/null
nohup cargo run > /tmp/rustmp3.log 2>&1 &
echo "Running rust_mp3 on port $PORT"
