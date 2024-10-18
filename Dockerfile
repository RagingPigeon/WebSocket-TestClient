FROM rust:latest as builder

# Set the working directory within the container.
WORKDIR /usr/src/app

ENV CARGO_HOME=/usr/local/cargo
ENV PATH=/usr/local/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin

# Copy everything from the source repository folder to the current
# location in the container.
COPY . .
RUN cargo install --path .

# Build the "deployment" image using a Read Hat image, which maches the
# images used in Iron Bank.
# Building a second image allows us to include only what we need to actually
# run the application.  All the intermediate build artifacts will be
# left out of this image.
FROM redhat/ubi9:9.4

# Copy the application from the build image.
COPY --from=builder /usr/local/cargo/bin/WebSocket-TestClient /usr/local/bin/WebSocket-TestClient

# The Rust program name is defined in the Cargo.toml file.
CMD ["WebSocket-TestClient"]