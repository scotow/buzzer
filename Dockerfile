FROM rust:1.73-slim AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

#------------

FROM gcr.io/distroless/cc-debian12

COPY --from=builder /app/target/release/buzzer /buzzer

ENTRYPOINT ["/buzzer"]