#!/usr/bin/env bash
set -eo pipefail

check_command() {
  if ! command -v "$1" &> /dev/null; then
    echo >&2 "Error: $1 is not installed."
    exit 1
  fi
}

check_command "psql"
check_command "diesel"

# Define your local PostgreSQL credentials
DB_USER=${POSTGRES_USER:=postgres}
DB_PASSWORD="${POSTGRES_PASSWORD:=mysecretpassword}"
DB_NAME="${POSTGRES_DB:=payments_db}"
DB_HOST="${POSTGRES_HOST:=localhost}"
DB_PORT="${POSTGRES_PORT:=5432}"

# Function to wait for PostgreSQL to be ready
wait_for_postgres() {
  echo "Waiting for PostgreSQL to start..."
  until PGPASSWORD="${DB_PASSWORD}" psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
    >&2 echo "Postgres is still unavailable - sleeping"
    sleep 1
  done
  echo "PostgreSQL is up and running on ${DB_HOST}:${DB_PORT}!"
}

# Set up PostgreSQL and run migrations using Diesel
setup_database() {
  echo "Setting up database..."
  export DATABASE_URL=postgresql://"${DB_USER}":"${DB_PASSWORD}"@"${DB_HOST}":"${DB_PORT}"/"${DB_NAME}"
  diesel setup
  diesel migration run
}

# Main function to orchestrate setup and application start
main() {
  wait_for_postgres
  setup_database
  echo "Postgres has been migrated and setup, ready to start the application!"
}

# Run main function
main
