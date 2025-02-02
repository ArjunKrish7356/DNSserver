# Stage 1: Build the application
FROM rust:1.75-slim as builder

WORKDIR /usr/src/app
COPY . .

# Build the async DNS resolver
RUN cargo build --release --bin asyncDnsResolver

# Stage 2: Create the runtime image
FROM debian:bookworm-slim

WORKDIR /usr/local/bin

# Copy the built binary
COPY --from=builder /usr/src/app/target/release/asyncDnsResolver .

# Expose DNS ports
EXPOSE 53/udp
EXPOSE 2053/udp

# Run the DNS server
CMD ["./asyncDnsResolver"]