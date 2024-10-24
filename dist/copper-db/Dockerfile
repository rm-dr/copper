FROM postgres:latest

# This is not secure, but this isn't an issue.
# Our postgres server should never be exposed to the internet.
ENV POSTGRES_USER="admin"
ENV POSTGRES_PASSWORD="admin"

COPY ./init_db.sh /docker-entrypoint-initdb.d/10-init_db.sh
