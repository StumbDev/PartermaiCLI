# Base image
FROM rust:latest

# Install required packages
RUN apt-get update && \
    apt-get install -y build-essential

# Create working directory
WORKDIR /app

# Copy code to container
COPY . .

# Build the Rust project
RUN cargo build --release

# Run the application
CMD ["cargo", "run", "--release"]
