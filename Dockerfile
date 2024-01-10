####################################################################################################
## Builder
####################################################################################################
FROM rust:latest AS builder

# Create appuser
ENV USER=appuser
ENV UID=10001

RUN adduser \
  --disabled-password \
  --gecos "" \
  --home "/nonexistent" \
  --shell "/sbin/nologin" \
  --no-create-home \
  --uid "${UID}" \
  "${USER}"

WORKDIR /app

COPY ./Cargo.toml ./Cargo.toml
COPY ./shared ./shared
COPY ./shared_types ./shared_types
COPY ./server ./server

WORKDIR /app/server
RUN cargo build --release

####################################################################################################
## Final image
####################################################################################################
FROM debian:bookworm-slim

RUN apt-get update && apt-get install libssl3 ca-certificates -y && rm -rf /var/lib/apt/lists/*
RUN update-ca-certificates

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /app

# Copy our build
COPY --from=builder /app/target/release/animal-hunt-server ./

# Use an unprivileged user.
USER appuser:appuser

EXPOSE 8080
CMD ["/app/animal-hunt-server"]
