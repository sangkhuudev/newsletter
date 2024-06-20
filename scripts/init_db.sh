#!/usr/bin/env bash
set -x 
set -eo pipefail

# Check if a custom user has been set, otherwise set default to 'postgres'
DB_USER="${POSTGRES_USER:=postgres}"

# Check if a custom password has been set, otherwise set default to '123456'"
DB_PASSWORD="${POSTGRES_PASSWORD:=123456}"

# Check if a custom database has been set, otherwise set default to 'newsletter'
DB_NAME="${POSTGRES_DB:=newsletter}"

# Check if a custom port has been set, otherwise set default to '5432'
DB_PORT="${POSTGRES_PORT:=5432}"

# Check if a custom host has been set, otherwise set default to 'localhost'
DB_HOST="${POSTGRES_HOST:=localhost}"

# Check if postgres CLI is installed
if ! [ -x "$(command -v psql)" ]; then
    echo >&2 "Error: psql is not installed."
    exit 1
fi

# Check if sqlx CLI is installed
if ! [ -x "$(command -v sqlx)" ]; then
    echo >&2 "Error: sqlx is not installed."
    echo >&2 "Use:"
    echo >&2 "cargo install sqlx-cli --no-default-features --features rustls,postgres"
    echo >&2 "to install it."
    exit 1
fi


# Allow to skip Docker if a dockerized Postgres database is already running
if [[ -z "${SKIP_DOCKER}" ]]
then
  # if a postgres container is running, print instructions to kill it and exit
  RUNNING_POSTGRES_CONTAINER=$(docker ps --filter 'name=postgres' --format '{{.ID}}')
  if [[ -n $RUNNING_POSTGRES_CONTAINER ]]; then
    echo >&2 "there is a postgres container already running, kill it with"
    echo >&2 "    docker kill ${RUNNING_POSTGRES_CONTAINER}"
    exit 1
  fi
  # Launch postgres using Docker
  docker run \
      -e POSTGRES_USER=${DB_USER} \
      -e POSTGRES_PASSWORD=${DB_PASSWORD} \
      -e POSTGRES_DB=${DB_NAME} \
      -p "${DB_PORT}":5432 \
      -d \
      --name "postgres_$(date '+%s')" \
      postgres -N 1000
      # ^ Increased maximum number of connections for testing purposes
fi


# Check if Docker started the container successfully
if [ $? -ne 0 ]; then
    echo >&2 "Error: Failed to start PostgreSQL container."
    exit 1
fi

# Keep pinging Postgres until it's ready to accept commands
export PGPASSWORD="${DB_PASSWORD}"
until psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
    >&2 echo "Postgres is still unavailable - sleeping"
    sleep 1
done

>&2 echo "Postgres is up and running on port ${DB_PORT} - running migrations now!"

# Set up the database URL
DATABASE_URL="postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}"
export DATABASE_URL
echo "DATABASE_URL is set to: $DATABASE_URL"

# Create the database and migrate (sqlx docs requires DATABASE_URL exists) 
sqlx database create
sqlx migrate run

>&2 echo "Postgres has been migrated, ready to go!"