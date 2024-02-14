# redis-logger

[![Crates.io](https://img.shields.io/crates/v/redis-logger.svg)](https://crates.io/crates/redis-logger)
[![Docs](https://docs.rs/redis-logger/badge.svg)](https://docs.rs/redis-logger)
[![MIT/APACHE-2.0](https://img.shields.io/crates/l/redis-logger.svg)](https://crates.io/crates/redis-logger)

This `log::Log` implementation allows to log to a Redis server. It supports writing to any number of pub/sub channels or streams or both.

You can specify custom encoders for pub/sub and stream log messages. Using the `default_encoding` feature default implementations for the encoders are available. This feature is disabled by default.

## Usage

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
log = "0.4"
redis = "0.24"
redis_logger = "0.2"
```

How to use in your application:

Build a `RedisConfig` using the `RedisConfig::builder()` call. Specify a connection and at least one pub/sub or stream channel. Use this configuration to either instantiate a `RedisLogger` instance with `RedisLogger::new` if you wish to use this logger with other loggers (like the [threaded_proxy_logger](https://crates.io/crates/threaded_proxy_logger) crate or [CombinedLogger](https://crates.io/crates/simplelog) logger from the `simplelog` crate) or use the `RedisLogger::init` method to initialize the logger as the only logger for the application.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

