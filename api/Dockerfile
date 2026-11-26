# Build with the Rust toolchain, then ship only the binary: no compiler, no shell, non-root.
FROM rust:1.97 AS build
WORKDIR /src
COPY . .
RUN cargo build --release --locked

FROM gcr.io/distroless/cc-debian13:nonroot
COPY --from=build /src/target/release/bookmarks-api /api
ENTRYPOINT ["/api"]
