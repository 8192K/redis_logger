#![cfg_attr(docsrs, feature(doc_cfg))]

//! # Redis Logger
//!
//! This module provides a logger implementation that logs messages to Redis using the `log` crate's `Log` trait.
//!
//! ## `RedisLogger`
//!
//! `RedisLogger` is the main struct in this module. It implements the `Log` trait, which allows it to be used as a logger in
//!  applications that use the `log` crate. It logs messages to Redis,
//!  either by publishing them to any number of pub/sub channels or by adding them to streams or both.
//!
//! ## `RedisLoggerConfig`
//!
//! `RedisLoggerConfig` is a struct that holds the configuration for a `RedisLogger`.
//!  It includes a Redis connection, a list of pub/sub channels and/or a list of streams to log to,
//!  along with encoders for the messages.
//!
//! ## `RedisLoggerConfigBuilder`
//!
//! `RedisLoggerConfigBuilder` is a builder for `RedisLoggerConfig`. It provides a fluent interface for building a `RedisLoggerConfig`.
//!
//! ## `PubSubEncoder` and `StreamEncoder`
//!
//! `PubSubEncoder` and `StreamEncoder` are traits for encoding log messages.
//! They are used by `RedisLogger` to encode the messages before sending them to Redis.
//! The module provides default implementations of these traits when the feature `default_encoders` is enabled,
//! but users can also provide their own implementations.
//!
//! ## Usage
//!
//! To use this logger, you need to create a `RedisLoggerConfig` (using `RedisLoggerConfigBuilder`), create a `RedisLogger` with the config,
//! either by calling `::new` or `::init`, the latter of which also sets the logger as the global logger.
//!
//! We recommend using this logger with the `parallel_logger` crate to avoid blocking the main thread when logging to Redis.
//!
//! ## Example
//! This example shows how to implement a `PubSubEncoder` that encodes log messages as a byte vector using the `bincode` crate. It also
//! shows how to configure `RedisLogger` to use this encoder while being part of multiple loggers that run on a separate thread using `parallel_logger`.
//! ```rust,ignore
//! struct BincodeRedisEncoder;
//! 
//! impl PubSubEncoder for BincodeRedisEncoder {
//!     fn encode(&self, record: &log::Record) -> Vec<u8> {
//!         let mut slice = [0u8; 2000];
//!         let message = SerializableLogRecord::from(record);
//!         let size = bincode::encode_into_slice(message, &mut slice, BINCODE_CONFIG).unwrap();
//!         let slice = &slice[..size];
//!         slice.to_vec()
//!     }
//! }
//! 
//! fn main() {
//!     let redis_client = redis::Client::open(REDIS_URL).unwrap();
//!     let redis_connection = redis_client.get_connection().unwrap();
//! 
//!     ParallelLogger::init(
//!         log::LevelFilter::Debug,
//!         ParallelMode::Sequential,
//!         vec![
//!             FileLogger::new(LevelFilter::Debug, "log_file.log"),
//!             TerminalLogger::new(LevelFilter::Info),
//!             RedisLogger::new(
//!                 LevelFilter::Debug,
//!                 RedisLoggerConfigBuilder::build_with_pubsub(
//!                     redis_connection,
//!                     vec!["logging".into()],
//!                     BincodeRedisEncoder {},
//!                 ),
//!             ),
//!         ],
//!     );
//! }
//! ```
//! Using `RedisLogger::init` insted of `RedisLogger::new` would allow the logger to be used as the only global logger.
//!
//! ## Features
//!
//! This module has a feature flag `default_encoders` that, when enabled, provides default implementations
//! of `PubSubEncoder` and `StreamEncoder` that encode the log messages as JSON or as a vector of tuples, respectively.
//!
//! Another feature flag `shared_logger` implements the `simplelog::SharedLogger` trait for `RedisLogger`. This enables use in a `simplelog::CombinedLogger`.

use std::sync::Mutex;

use log::{LevelFilter, Log, Metadata, Record, SetLoggerError};
use redis::ConnectionLike;

#[cfg_attr(docsrs, doc(cfg(feature = "default_encoders")))]
#[cfg(feature = "default_encoders")]
mod defaults;
#[cfg(feature = "default_encoders")]
pub use defaults::*;

#[cfg(test)]
mod lib_tests;

/// Trait for encoding log messages to be published to a pub/sub channel.
pub trait PubSubEncoder: Send + Sync + Sized {
    /// Encodes the given `log::Record` into a byte vector.
    fn encode(&self, record: &Record) -> Vec<u8>;
}

/// Trait for encoding log messages to be added to a Redis stream.
pub trait StreamEncoder: Send + Sync + Sized {
    /// Encodes the given `log::Record` into a vector of tuples of a field name and the corresponding value as a byte vector.
    fn encode(&self, record: &Record) -> Vec<(String, Vec<u8>)>;
}

/// Placeholder. Cannot be instantiated or used. Necessary as a placeholder when not specifing a pub/sub encoder.
#[derive(Debug)]
#[doc(hidden)]
#[non_exhaustive]
pub struct DummyPubSubEncoder {}

#[doc(hidden)]
impl PubSubEncoder for DummyPubSubEncoder {
    fn encode(&self, _record: &Record) -> Vec<u8> {
        panic!()
    }
}

/// Placeholder. Cannot be instantiated or used. Necessary as a placeholder when not specifing a stream encoder.
#[derive(Debug)]
#[doc(hidden)]
#[non_exhaustive]
pub struct DummyStreamEncoder {}

#[doc(hidden)]
impl StreamEncoder for DummyStreamEncoder {
    fn encode(&self, _record: &Record) -> Vec<(String, Vec<u8>)> {
        panic!()
    }
}

#[derive(Debug)]
/// A logger that logs messages to Redis.
pub struct RedisLogger<CONN, PUBSUB, STREAM>
where
    CONN: ConnectionLike + Send + Sync,
    PUBSUB: PubSubEncoder,
    STREAM: StreamEncoder,
{
    level: LevelFilter,
    config: RedisLoggerConfig<CONN, PUBSUB, STREAM>,
}

impl<CONN, PUBSUB, STREAM> RedisLogger<CONN, PUBSUB, STREAM>
where
    CONN: ConnectionLike + Send + Sync + 'static,
    PUBSUB: PubSubEncoder + 'static,
    STREAM: StreamEncoder + 'static,
{
    /// Creates a new instance of `RedisLogger` with the specified log level and configuration.
    ///
    /// # Arguments
    ///
    /// * `level` - The log level to set for the logger.
    /// * `config` - The configuration for the Redis logger.
    ///
    /// # Returns
    ///
    /// A boxed instance of `RedisLogger`, not yet initialized as the global logger.
    pub fn new(level: LevelFilter, config: RedisLoggerConfig<CONN, PUBSUB, STREAM>) -> Box<Self> {
        Box::new(Self { level, config })
    }

    /// Initializes the Redis logger with the specified log level and configuration.
    ///
    /// # Arguments
    ///
    /// * `level` - The log level to set for the logger.
    /// * `config` - The configuration for the Redis logger.
    ///
    /// # Returns
    ///
    /// Result indicating success or an error of type `RedisLoggerConfigError`.
    /// If successful, the logger is set as the global logger.
    ///
    /// # Errors
    ///
    /// see above
    pub fn init(level: LevelFilter, config: RedisLoggerConfig<CONN, PUBSUB, STREAM>) -> Result<(), SetLoggerError> {
        let redis_logger = Self::new(level, config);
        log::set_max_level(level);
        log::set_boxed_logger(redis_logger)?;
        Ok(())
    }
}

/// Implements the `Log` trait for the `RedisLogger` struct.
///
/// This implementation provides the necessary methods to enable logging to Redis.
/// The `enabled` method checks if the log level of the provided `Metadata` is less than or equal to the configured log level.
/// The `log` method publishes log messages to Redis channels and streams based on the configuration in one atomic operation using a pipeline.
/// The `flush` method is a no-op in this implementation.
impl<CONN, PUBSUB, STREAM> Log for RedisLogger<CONN, PUBSUB, STREAM>
where
    CONN: ConnectionLike + Send + Sync,
    PUBSUB: PubSubEncoder,
    STREAM: StreamEncoder,
{
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let config = &self.config;
            let mut pipe = redis::pipe();
            if let Some((channels, encoder)) = &config.channels {
                let message = encoder.encode(record);
                for channel in channels {
                    pipe.publish(channel, &message);
                }
            }
            if let Some((streams, encoder)) = &config.streams {
                let message = encoder.encode(record);
                let message = message.as_slice();
                for stream in streams {
                    pipe.xadd(stream, "*", message);
                }
            }

            // this unwrap only panics if the connection is poisoned, so we can't do much anyway and will panic, too!
            if let Err(e) = pipe.query::<()>(&mut config.connection.lock().unwrap()) {
                eprintln!("Error logging to Redis: {e}");
            }
        }
    }

    fn flush(&self) {}
}

/// Configuration for the Redis logger. Pass to `RedisLogger` to configure the logger.
#[derive(Debug)]
pub struct RedisLoggerConfig<CONN, PUBSUB, STREAM>
where
    CONN: ConnectionLike + Send + Sync,
    PUBSUB: PubSubEncoder,
    STREAM: StreamEncoder,
{
    connection: Mutex<CONN>,
    channels: Option<(Vec<String>, PUBSUB)>,
    streams: Option<(Vec<String>, STREAM)>,
}

/// `RedisLoggerConfigBuilder` is a builder for `RedisLoggerConfig`.
///  
/// # Panics
///
/// Panics if the channels or streams vectors are empty when building the `RedisLoggerConfig`.
#[derive(Debug)]
#[non_exhaustive]
pub struct RedisLoggerConfigBuilder {}

impl RedisLoggerConfigBuilder {
    /// Constructs a `RedisLoggerConfig` with a given connection, channels, and a Pub/Sub encoder.
    ///
    /// # Arguments
    ///
    /// * `connection` - A connection that implements `ConnectionLike + Send + Sync`.
    /// * `channels` - A vector of channel names.
    /// * `encoder` - An encoder that implements `PubSubEncoder`.
    ///
    /// # Returns
    ///
    /// A `RedisLoggerConfig` with the given connection, channels, and Pub/Sub encoder.
    ///
    /// # Panics
    ///
    /// Panics if the channels vector is empty
    pub fn build_with_pubsub<CONN, PUBSUB>(
        connection: CONN,
        channels: Vec<String>,
        encoder: PUBSUB,
    ) -> RedisLoggerConfig<CONN, PUBSUB, DummyStreamEncoder>
    where
        CONN: ConnectionLike + Send + Sync,
        PUBSUB: PubSubEncoder,
    {
        Self::check_args(!channels.is_empty());
        RedisLoggerConfig {
            connection: Mutex::new(connection),
            channels: Some((channels, encoder)),
            streams: None,
        }
    }

    /// Constructs a `RedisLoggerConfig` with a given connection and channels, using the default Pub/Sub encoder.
    ///
    /// This method is only available when the `default_encoders` feature is enabled.
    ///
    /// # Arguments
    ///
    /// * `connection` - A connection that implements `ConnectionLike + Send + Sync`.
    /// * `channels` - A vector of channel names.
    ///
    /// # Returns
    ///
    /// A `RedisLoggerConfig` with the given connection and channels, and the default Pub/Sub encoder.
    ///
    /// # Panics
    ///
    /// Panics if the channels vector is empty
    #[cfg(feature = "default_encoders")]
    pub fn build_with_pubsub_default<CONN>(
        connection: CONN,
        channels: Vec<String>,
    ) -> RedisLoggerConfig<CONN, DefaultPubSubEncoder, DummyStreamEncoder>
    where
        CONN: ConnectionLike + Send + Sync,
    {
        Self::check_args(!channels.is_empty());
        RedisLoggerConfig {
            connection: Mutex::new(connection),
            channels: Some((channels, DefaultPubSubEncoder::new())),
            streams: None,
        }
    }

    /// Constructs a `RedisLoggerConfig` with a given connection, streams, and a Stream encoder.
    ///
    /// # Arguments
    ///
    /// * `connection` - A connection that implements `ConnectionLike + Send + Sync`.
    /// * `streams` - A vector of stream names.
    /// * `encoder` - An encoder that implements `StreamEncoder`.
    ///
    /// # Returns
    ///
    /// A `RedisLoggerConfig` with the given connection, streams, and Stream encoder.
    ///
    /// # Panics
    ///
    /// Panics if the streams vector is empty
    pub fn build_with_streams<CONN, STREAM>(
        connection: CONN,
        streams: Vec<String>,
        encoder: STREAM,
    ) -> RedisLoggerConfig<CONN, DummyPubSubEncoder, STREAM>
    where
        CONN: ConnectionLike + Send + Sync,
        STREAM: StreamEncoder,
    {
        Self::check_args(!streams.is_empty());
        RedisLoggerConfig {
            connection: Mutex::new(connection),
            channels: None,
            streams: Some((streams, encoder)),
        }
    }

    /// Constructs a `RedisLoggerConfig` with a given connection and streams, using the default Stream encoder.
    ///
    /// This method is only available when the `default_encoders` feature is enabled.
    ///
    /// # Arguments
    ///
    /// * `connection` - A connection that implements `ConnectionLike + Send + Sync`.
    /// * `streams` - A vector of stream names.
    ///
    /// # Returns
    ///
    /// A `RedisLoggerConfig` with the given connection and streams, and the default Stream encoder.
    ///
    /// # Panics
    ///
    /// Panics if the streams vector is empty
    #[cfg(feature = "default_encoders")]
    pub fn build_with_streams_default<CONN>(
        connection: CONN,
        streams: Vec<String>,
    ) -> RedisLoggerConfig<CONN, DummyPubSubEncoder, DefaultStreamEncoder>
    where
        CONN: ConnectionLike + Send + Sync,
    {
        Self::check_args(!streams.is_empty());
        RedisLoggerConfig {
            connection: Mutex::new(connection),
            channels: None,
            streams: Some((streams, DefaultStreamEncoder::new())),
        }
    }

    /// Constructs a `RedisLoggerConfig` with a given connection, channels, streams, a Pub/Sub encoder, and a Stream encoder.
    ///
    /// # Arguments
    ///
    /// * `connection` - A connection that implements `ConnectionLike + Send + Sync`.
    /// * `channels` - A vector of channel names.
    /// * `pubsub_encoder` - An encoder that implements `PubSubEncoder`.
    /// * `streams` - A vector of stream names.
    /// * `stream_encoder` - An encoder that implements `StreamEncoder`.
    ///
    /// # Returns
    ///
    /// A `RedisLoggerConfig` with the given connection, channels, streams, Pub/Sub encoder, and Stream encoder.
    ///
    /// # Panics
    ///
    /// Panics if the streams and channels vectors are both empty
    pub fn build_with_pubsub_and_streams<CONN, PUBSUB, STREAM>(
        connection: CONN,
        channels: Vec<String>,
        pubsub_encoder: PUBSUB,
        streams: Vec<String>,
        stream_encoder: STREAM,
    ) -> RedisLoggerConfig<CONN, PUBSUB, STREAM>
    where
        CONN: ConnectionLike + Send + Sync,
        PUBSUB: PubSubEncoder,
        STREAM: StreamEncoder,
    {
        Self::check_args(!channels.is_empty() && !streams.is_empty());
        RedisLoggerConfig {
            connection: Mutex::new(connection),
            channels: Some((channels, pubsub_encoder)),
            streams: Some((streams, stream_encoder)),
        }
    }

    /// Constructs a `RedisLoggerConfig` with a given connection, channels, and streams, using the default Pub/Sub and Stream encoders.
    ///
    /// This method is only available when the `default_encoders` feature is enabled.
    ///
    /// # Arguments
    ///
    /// * `connection` - A connection that implements `ConnectionLike + Send + Sync`.
    /// * `channels` - A vector of channel names.
    /// * `streams` - A vector of stream names.
    ///
    /// # Returns
    ///
    /// A `RedisLoggerConfig` with the given connection, channels, streams, and the default Pub/Sub and Stream encoders.
    ///
    /// # Panics
    ///
    /// Panics if the streams and channels vectors are both empty
    #[cfg(feature = "default_encoders")]
    pub fn build_with_pubsub_and_streams_default<CONN>(
        connection: CONN,
        channels: Vec<String>,
        streams: Vec<String>,
    ) -> RedisLoggerConfig<CONN, DefaultPubSubEncoder, DefaultStreamEncoder>
    where
        CONN: ConnectionLike + Send + Sync,
    {
        Self::check_args(!channels.is_empty() && !streams.is_empty());
        RedisLoggerConfig {
            connection: Mutex::new(connection),
            channels: Some((channels, DefaultPubSubEncoder::new())),
            streams: Some((streams, DefaultStreamEncoder::new())),
        }
    }

    const fn check_args(value: bool) {
        assert!(
            value,
            "Channels not set in RedisLogger. Set at least one pub/sub channel and/or one stream channel."
        );
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "shared_logger")))]
#[cfg(feature = "shared_logger")]
impl<CONN, PUBSUB, STREAM> simplelog::SharedLogger for RedisLogger<CONN, PUBSUB, STREAM>
where
    CONN: ConnectionLike + Send + Sync + 'static,
    PUBSUB: PubSubEncoder + 'static,
    STREAM: StreamEncoder + 'static,
{
    fn level(&self) -> log::LevelFilter {
        self.level
    }

    fn config(&self) -> Option<&simplelog::Config> {
        None
    }

    fn as_log(self: Box<Self>) -> Box<dyn Log> {
        self
    }
}
