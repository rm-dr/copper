#!/bin/bash
set -eu

POSTGRES="psql --username ${POSTGRES_USER}"

# This is not secure, but this isn't an issue.
# Our postgres server should never be exposed to the internet.
EDGED_PASSWORD="edged"
STORAGED_PASSWORD="storaged"

echo "Initializing edged"
$POSTGRES <<-EOSQL
CREATE USER edged WITH CREATEDB PASSWORD '${EDGED_PASSWORD}';
CREATE DATABASE edged OWNER edged;
EOSQL

echo "Initializing storaged"
$POSTGRES <<-EOSQL
CREATE USER storaged WITH CREATEDB PASSWORD '${STORAGED_PASSWORD}';
CREATE DATABASE storaged OWNER storaged;
EOSQL
