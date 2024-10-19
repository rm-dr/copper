#
# MARK: builder
#
FROM rust:1.82.0-alpine AS builder

RUN apk update && \
	apk upgrade && \
	apk add musl-dev openssl-dev ca-certificates && \
	update-ca-certificates

# This isn't found automatically,
# not sure why
ENV OPENSSL_DIR=/usr

WORKDIR /copperd
COPY . /copperd

# Compile with musl for fully static linking
RUN rustup target add x86_64-unknown-linux-musl && \
	cargo build --workspace --target=x86_64-unknown-linux-musl --release



#
# MARK: final container
#
FROM alpine:3.20
ARG version=unknown
ARG release=unreleased
LABEL name="Copper" \
	maintainer="mark@betalupi.com" \
	vendor="Copper" \
	version=${version} \
	release=${release} \
	summary="The universal, automatic digital library" \
	description="The universal, automatic digital library"

# We could do this though `apk`, but this produces a smaller image.
# It also lets us easily move to `FROM scratch` instead of `FROM alpine`...
#
# We build from alpine here only because we need `sh` to run all of copper's
# daemon's in parallel. TODO: find a simple way to do this `FROM scratch`.
VOLUME [ "/certs" ]
ENV SSL_CERT_FILE="/certs/ca-certificates.crt"
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /certs/ca-certificates.crt

ARG builder_source="/copperd/target/x86_64-unknown-linux-musl/release"
COPY --from=builder \
	${builder_source}/pipelined \
	${builder_source}/storaged \
	${builder_source}/edged \
	/


#
# Set up env vars
#

# These vars are only used for substitutions.
ENV SERVER_ADDR "80"
ENV PIPELINED_PORT "1000"
ENV STORAGED_PORT "2000"
EXPOSE 80/tcp

# Users should not modify any variables below.
ENV EDGED_SERVER_ADDR "0.0.0.0:${SERVER_ADDR}"
# These are bad secrets, but it's ok:
# pipelined and storaged are bound to 127.0.0.1,
# and thus only receive local connections.
ENV PIPELINED_SERVER_ADDR "127.0.0.1:${PIPELINED_PORT}"
ENV PIPELINED_SECRET "pipelined_secret"
ENV STORAGED_SERVER_ADDR "127.0.0.1:${STORAGED_PORT}"
ENV STORAGED_SECRET "storaged_secret"

ENV EDGED_STORAGED_ADDR "http://127.0.0.1:${STORAGED_PORT}"
ENV EDGED_STORAGED_SECRET "${STORAGED_SECRET}"
ENV EDGED_PIPELINED_ADDR "http://127.0.0.1:${PIPELINED_PORT}"
ENV EDGED_PIPELINED_SECRET "${PIPELINED_SECRET}"
ENV PIPELINED_STORAGED_ADDR "http://127.0.0.1:${STORAGED_PORT}"
ENV PIPELINED_STORAGED_SECRET "${STORAGED_SECRET}"

ENTRYPOINT ["/bin/sh", "-c", "./pipelined & ./storaged & ./edged"]
