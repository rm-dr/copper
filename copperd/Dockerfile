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
	${builder_source}/edged \
	/


#
# Set up env vars
#

# These vars are only used for substitutions.
ENV SERVER_ADDR "80"
EXPOSE 80/tcp
ENV EDGED_SERVER_ADDR "0.0.0.0:${SERVER_ADDR}"

ENTRYPOINT ["/bin/sh", "-c", "./pipelined & ./edged"]
