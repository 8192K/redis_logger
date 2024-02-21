# redis_logger

[![Crates.io](https://img.shields.io/crates/v/redis_logger.svg)](https://crates.io/crates/redis_logger)
[![Docs](https://docs.rs/redis_logger/badge.svg)](https://docs.rs/redis_logger)
[![MIT/APACHE-2.0](https://img.shields.io/crates/l/redis_logger.svg)](https://crates.io/crates/redis_logger)

This `log::Log` implementation allows to log to a Redis server. It supports writing to any number of pub/sub channels or streams or both.

You can specify custom encoders for pub/sub and stream log messages. Using the `default_encoders` feature default implementations for the encoders are available. This feature is disabled by default.

If you enable the `shared_logger` feature you can use the `RedisLogger` inside a `simplelog::CombinedLogger`.

## Usage

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
log = "0.4"
redis = "0.24"
redis_logger = "0.3"
```

How to use in your application:

Build a `RedisLoggerConfig` using the `RedisLoggerConfigBuilder` methods. Specify a connection and at least one pub/sub or stream channel. Use this configuration to either instantiate a `RedisLogger` instance with `RedisLogger::new` if you wish to use this logger with other loggers (like the [parallel_logger](https://crates.io/crates/parallel_logger) crate or [CombinedLogger](https://crates.io/crates/simplelog) logger from the `simplelog` crate) or use the `RedisLogger::init` method to initialize the logger as the only logger for the application.

A simple example using the `default_encoders` feature and setting the `RedisLogger` as the only logger would look like this:
```rust
fn main() {
    let redis_client = redis::Client::open(REDIS_URL).unwrap();
    let redis_connection = redis_client.get_connection().unwrap();

    RedisLogger::init(
        LevelFilter::Debug,
        RedisLoggerConfigBuilder::build_with_pubsub_default(
            redis_connection,
            vec!["logging".into()],
        ),
    );
}
```

This broader example uses `RedisLogger` inside a `ParallelLogger` and encodes messages for pub/sub using the `bincode` crate and a custom `PubSubEncoder`:
```rust
struct BincodeRedisEncoder;

impl PubSubEncoder for BincodeRedisEncoder {
    fn encode(&self, record: &log::Record) -> Vec<u8> {
        let mut slice = [0u8; 2000];
        let message = SerializableLogRecord::from(record);
        let size = bincode::encode_into_slice(message, &mut slice, BINCODE_CONFIG).unwrap();
        let slice = &slice[..size];
        slice.to_vec()
    }
}
 
fn main() {
    let redis_client = redis::Client::open(REDIS_URL).unwrap();
    let redis_connection = redis_client.get_connection().unwrap();

    ParallelLogger::init(
        log::LevelFilter::Debug,
        ParallelMode::Sequential,
        vec![
            FileLogger::new(LevelFilter::Debug, "log_file.log"),
            TerminalLogger::new(LevelFilter::Info),
            RedisLogger::new(
                LevelFilter::Debug,
                RedisLoggerConfigBuilder::build_with_pubsub(
                    redis_connection,
                    vec!["logging".into()],
                    BincodeRedisEncoder {},
                ),
            ),
        ],
    );
}
```

## Roadmap

- Support other Redis crates than `redis_rs` (like `fred`).
- Support atomic pipelines when calling Redis.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

