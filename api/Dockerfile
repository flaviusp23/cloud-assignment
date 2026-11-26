# ---------- Stage 1: build ----------
FROM rust:1.97 AS build
WORKDIR /src

# Dependencies first: compile them against an empty main, so this layer is
# cached until Cargo.toml/Cargo.lock change.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs \
    && cargo build --release --locked \
    && rm -rf src

# Then the code, which changes often.
COPY src ./src
RUN touch src/main.rs && cargo build --release --locked

# ---------- Stage 2: run ----------
# No compiler, no shell, runs as a non-root user.
FROM gcr.io/distroless/cc-debian13:nonroot
COPY --from=build /src/target/release/bookmarks-api /api
EXPOSE 8080
ENTRYPOINT ["/api"]
