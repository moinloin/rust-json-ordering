FROM rust:latest

WORKDIR /app

# Copy Cargo.toml and Cargo.lock
COPY Cargo.toml Cargo.lock* ./

# Create empty source file to build dependencies
RUN mkdir -p src && echo "fn main() {println!(\"Hello\");}" > src/main.rs

# Build dependencies
RUN cargo build

# Remove the dummy source file
RUN rm -f src/main.rs

# Copy the actual source code
COPY src ./src

# Copy the JSON file
COPY json.txt ./

# Run the application
CMD ["cargo", "run"]