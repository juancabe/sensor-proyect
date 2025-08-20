# Sensor Server

Rust implementation for the sernsors server

## TODOs

- [x] Methods that should accept url params aren't
- [x] Refactor so that API Endpoints return API Path
- [x] Add better log information for the endpoints
- [x] Export bindings to TS
- [ ] Add Put endpoints where needed
- [ ] Use PoisonedIdentifiers correctly
- [ ] Try to refactor DB to LazyLock<Mutex<...>> or something to share a DbConn on the state (maybe no)
- [ ] Impose restrictions on user-input fields

## Main dependencies

### Services used

- PostgreSQL

### Libraries used

- diesel-rs: database ORM
- axum: providing HTTP server abstractions and implementations

## How to setup

1. Install PostgreSQL for your system
2. Install diesel with PostgreSQL and configure it: [guide](https://diesel.rs/guides/getting-started)
3. Setup your TLS keys on the private/ dir
