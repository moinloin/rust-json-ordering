FROM rust:latest

WORKDIR /app

# Copy Cargo.toml and Cargo.lock
COPY Cargo.toml Cargo.lock* ./

# Create empty source file to build dependencies
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# Build dependencies
RUN cargo build

# Remove the dummy source file
RUN rm -rf src

# Copy the actual source code
COPY src ./src

# Run the application
CMD ["cargo", "run"]