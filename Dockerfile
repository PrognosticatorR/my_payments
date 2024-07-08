# Start from a base image with Rust and Diesel dependencies
FROM lukemathwalker/cargo-chef:latest-rust-latest as chef
WORKDIR /app
RUN apt update && apt install lld clang -y

# Separate stages for different Docker build steps
FROM chef as planner
COPY . .
# Compute a lock-like file for our project
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json
# Up to this point, if our dependency tree stays the same,
# all layers should be cached.
COPY . .
# Build our project
RUN cargo install diesel_cli --no-default-features --features postgres
RUN cargo build --release --bin my_payments

# Final stage for the runtime image
FROM debian:latest AS runtime
WORKDIR /app

# Install Diesel CLI, PostgreSQL client, and other dependencies
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends \
    openssl ca-certificates \
    libpq-dev \
    postgresql-client \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/my_payments my_payments

# Expose port 8080 if needed by your application
EXPOSE 8080

ENTRYPOINT [ "./my_payments" ]
