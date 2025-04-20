FROM rust:latest

WORKDIR /app

COPY Cargo.toml Cargo.lock* ./

RUN mkdir -p src && echo "fn main() {println!(\"Hello\");}" > src/main.rs

RUN cargo build

RUN rm -f src/main.rs

COPY src ./src

COPY json.txt ./

CMD ["cargo", "run"]