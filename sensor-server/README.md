# Sensor Server

Rust implementation for the sernsors server

## TODOs

- [x] Methods that should accept url params aren't
- [x] Refactor so that API Endpoints return API Path
- [x] Add better log information for the endpoints
- [x] Export bindings to TS
- [x] Add Put endpoints where needed
- [x] Use PoisonedIdentifiers correctly
- [x] Transform other api-input types into Validated types
- [x] Impose restrictions on user-input fields
- [ ] Test new Put endpoints
- [ ] Test PoisonedIdentifiers integration
- [ ] Test Api... types and PoisonedIdentifiers better

## Main dependencies

### Services used

- PostgreSQL

### Libraries used

- diesel-rs: database ORM
- axum: providing HTTP server abstractions and implementations
- and more

## How to setup

1. Install PostgreSQL for your system
2. Install diesel with PostgreSQL and configure it: [guide](https://diesel.rs/guides/getting-started)
3. Setup your TLS keys on the private/ dir
