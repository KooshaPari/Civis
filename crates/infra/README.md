# infra
> Shared infrastructure client wrappers for the Civis platform.

## Overview

The `infra` crate centralizes client-side access to Civis infrastructure services such as databases, message queues, key-value stores, and external API gateways. It provides thin async wrappers with automatic retries, circuit breaking, and observability hooks.

Every infrastructure client in the Civis stack depends on this crate for connection pooling, configuration loading, and graceful shutdown. This eliminates duplicated boilerplate and ensures consistent behavior across microservices.

The crate is designed for resilience. Connections are pooled and health-checked, with exponential backoff on transient failures. All client calls emit structured tracing spans for distributed observability.

## Features

- Async connection pooling with health checks
- Automatic retry with exponential backoff
- Circuit breaker pattern for fault tolerance
- Structured tracing on every client call
- Unified configuration from environment and config files
- Graceful shutdown and connection drain
- Feature-gated support for Postgres, Redis, and NATS

## Usage

```rust
use infra::{DbPool, Config};

let config = Config::from_env()?;
let pool = DbPool::connect(&config.database_url).await?;
let row = pool.query_one("SELECT 1").await?;
```

## Architecture

The crate exposes a `Client` trait that all infrastructure adapters implement. `Config` is deserialized from environment variables or a TOML file. Each adapter wraps its underlying driver in a pool manager that handles lifecycle, timeouts, and retry logic.

## License

Part of the Civis project (https://github.com/KooshaPari/Civis).
