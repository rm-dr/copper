FROM postgres:latest

ENV POSTGRES_USER "admin"
ENV POSTGRES_PASSWORD "admin"
COPY ./init_db.sh /docker-entrypoint-initdb.d/10-init_db.sh
