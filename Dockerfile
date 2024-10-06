FROM rust:1.80.1 as builder

WORKDIR /app

COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release

FROM rust:1.80.1-slim as runtime

WORKDIR /app
COPY --from=builder /app/target/release/rustquizbot rustquizbot
COPY --from=builder /app/.env .env
COPY --from=builder /app/migrations /migrations

RUN chmod +x rustquizbot

CMD [ "./rustquizbot" ]