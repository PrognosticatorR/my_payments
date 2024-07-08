#!/usr/bin/env bash
set -eo pipefail

check_command() {
  if ! command -v "$1" &> /dev/null; then
    echo >&2 "Error: $1 is not installed."
    exit 1
  fi
}

# Check necessary commands
check_command "docker"
check_command "docker-compose"

# Start the project using Docker Compose
echo "Starting the project with Docker Compose..."
docker-compose up -d || { echo >&2 "Failed to start Docker Compose."; exit 1; }

# Wait for the db service to be healthy
echo "Waiting for the database to be healthy..."

# Initialize the database if not already initialized
echo "Initializing the database..."
sh scripts/init_db.sh || { echo >&2 "Failed to initialize the database."; exit 1; }

echo "Setup complete! You can now access the APIs as described in the API documentation."
