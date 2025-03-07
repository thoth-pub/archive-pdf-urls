FROM clux/muslrust:1.85.0-stable as build

# Get source
COPY . .

# Build archive-pdf-urls for release
RUN cargo build --release

# Switch to minimal image for run time
FROM scratch

# Get archive-pdf-urls binary
COPY --from=build \
    /volume/target/x86_64-unknown-linux-musl/release/archive-pdf-urls /

# Get CA certificates
COPY --from=build \
    /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt


# Make archive-pdf-urls our default binary
ENTRYPOINT ["/archive-pdf-urls"]
