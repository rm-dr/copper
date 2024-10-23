#!/bin/bash
set -eu

POSTGRES="psql --username ${POSTGRES_USER}"

$POSTGRES <<-EOSQL
CREATE DATABASE edged;
CREATE DATABASE jobqueue;
CREATE DATABASE itemdb;
EOSQL

