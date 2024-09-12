FROM rust:latest

# Set the working directory within the container.
WORKDIR /usr/src/app

# Copy everything from the source repository folder to the current
# location in the container.
COPY . .

RUN cargo install --path .

# The Rust program name is defined in the Cargo.toml file.
CMD ["WebSocket-TestClient"]