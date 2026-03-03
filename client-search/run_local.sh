#!/bin/bash
set -e

# Run triton server
docker compose -f ../docker-compose.yml --profile client-search up -d

# Wait for triton server to be ready
while true; do
    status_code=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8000/v2/health/ready)
    if [ "$status_code" -eq 200 ]; then
        echo "Triton Server is ready!"
        break
    else
        echo "Triton Server not ready yet. Waiting..."
        sleep 2
    fi
done

# Define cleanup function
cleanup() {
    echo "Cleaning up..."
    if [ -n "$CARGO_PID" ] && kill -0 $CARGO_PID 2>/dev/null; then
        kill $CARGO_PID 2>/dev/null || true
        wait $CARGO_PID 2>/dev/null || true
    fi
    
    # Stop Triton Server
    docker compose -f ../docker-compose.yml --profile client-search down
}

# Trap SIGINT and SIGTERM
trap cleanup SIGINT SIGTERM EXIT

# Start Triton client application
export RUST_LOG=INFO
cd client && cargo run --release &
CARGO_PID=$!

# Wait for cargo process to finish and capture exit code
wait $CARGO_PID
CARGO_EXIT_CODE=$?

# If cargo crashed (non-zero exit), report it
if [ $CARGO_EXIT_CODE -ne 0 ]; then
    echo "Cargo process exited with code $CARGO_EXIT_CODE"
    exit $CARGO_EXIT_CODE
fi