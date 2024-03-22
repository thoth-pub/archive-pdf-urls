FROM clux/muslrust:1.76.0 as build

# Get source
COPY . .

# Build archive-pdf-urls for release
RUN cargo build --release

# Switch to minimal image for run time
FROM scratch

# Get archive-pdf-urls binary
COPY --from=build \
    /volume/target/x86_64-unknown-linux-musl/release/archive-pdf-urls /

# Make thoth our default binary
ENTRYPOINT ["/archive-pdf-urls"]
