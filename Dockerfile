FROM rust:1.73.0

WORKDIR /usr/src/sudoku

COPY . .

RUN cargo build --release

CMD ["./target/release/sudoku-rust"]
