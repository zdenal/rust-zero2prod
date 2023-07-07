## Why

To get more faimiliar with Rust language and stuff around like dockering application, CI, better error handling and so.

Based on this book: https://www.zero2prod.com/

## SQLX

To be able build docker image w/ release we need to have
sqlx feature 'offline' and run `cargo sqlx prepare` to generate
`sqlx-data.json` .. otherwise sqlx will raise a compile time error
related to no DB connection for checks.

## Docker

```
docker build --tag zero2prod --file Dockerfile .
docker run -p 8000:8000 zero2prod
```
