### Axum SQLx JWT Starter kit - WIP

Run locally

```
docker compose up
cargo watch -c -x check -x run
```

#### SQLx

```
cargo install sqlx-cli --no-default-features --features native-tls,postgres
sqlx migrate add <migration description>
sqlx migrate run
cargo sqlx prepare
```