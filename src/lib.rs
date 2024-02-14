//! # Lib Module
//!
//! This module provides a logger implementation that logs messages to Redis using the `log` crate's `Log` trait.
//!
//! ## `RedisLogger`
//!
//! `RedisLogger` is the main struct in this module. It implements the `Log` trait, which allows it to be used as a logger in
//!  applications that use the `log` crate. It logs messages to Redis,
//!  either by publishing them to a pub/sub channel or by adding them to a stream.
//!
//! ## `RedisLoggerConfig`
//!
//! `RedisLoggerConfig` is a struct that holds the configuration for a `RedisLogger`.
//!  It includes a Redis connection, and optionally a list of pub/sub channels and a list of streams to log to,
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
//! The module provides default implementations of these traits, but users can also provide their own implementations.
//!
//! ## Usage
//!
//! To use this logger, you need to create a `RedisLoggerConfig` (using `RedisLoggerConfigBuilder`), create a `RedisLogger` with the config,
//! either by calling `::new` or `::init`, the latter of which also sets the logger as the global logger.
//!
//! ## Features
//!
//! This module has a feature flag `default_encoding` that, when enabled, provides default implementations
//! of `PubSubEncoder` and `StreamEncoder` that encode the log messages as JSON or as a vector of tuples, respectively.

use std::sync::Mutex;

use log::{LevelFilter, Log, Metadata, Record};
use redis::ConnectionLike;

mod error;
pub use error::RedisLoggerConfigError;

#[cfg(feature = "default_encoding")]
mod defaults;
#[cfg(feature = "default_encoding")]
pub use defaults::*;

#[cfg(test)]
mod lib_tests;

/// Trait for encoding log messages to be published to a pub/sub channel.
pub trait PubSubEncoder: Send + Sync {
    /// Encodes the given `log::Record` into a byte vector.
    fn encode<'a>(&self, record: &Record<'a>) -> Vec<u8>;
    #[cfg(test)]
    fn name(&self) -> &'static str;
}

/// Trait for encoding log messages to be added to a Redis stream.
pub trait StreamEncoder: Send + Sync {
    /// Encodes the given `log::Record` into a vector of tuples of a field name and the corresponding value as a byte vector.
    fn encode<'a>(&self, record: &Record<'a>) -> Vec<(&'static str, Vec<u8>)>;
    #[cfg(test)]
    fn name(&self) -> &'static str;
}

pub trait RedisConnection: ConnectionLike + Send + Sync {
    fn as_mut_connection_like(&mut self) -> &mut dyn ConnectionLike;
}

impl RedisConnection for redis::Client {
    fn as_mut_connection_like(&mut self) -> &mut dyn ConnectionLike {
        self
    }
}

impl RedisConnection for redis::Connection {
    fn as_mut_connection_like(&mut self) -> &mut dyn ConnectionLike {
        self
    }
}

/// A logger that logs messages to Redis.
pub struct RedisLogger {
    level: LevelFilter,
    config: RedisLoggerConfig,
}

impl RedisLogger {
    /// Creates a new instance of RedisLogger with the specified log level and configuration.
    ///
    /// # Arguments
    ///
    /// * `level` - The log level to set for the logger.
    /// * `config` - The configuration for the Redis logger.
    ///
    /// # Returns
    ///
    /// A boxed instance of RedisLogger, not yet initialized as the global logger.
    pub fn new(level: LevelFilter, config: RedisLoggerConfig) -> Box<Self> {
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
    /// Result indicating success or an error of type RedisLoggerConfigError.
    /// If successful, the logger is set as the global logger.
    pub fn init(level: LevelFilter, config: RedisLoggerConfig) -> Result<(), RedisLoggerConfigError> {
        let redis_logger = RedisLogger::new(level, config);
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
impl Log for RedisLogger {
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
            if let Err(e) = pipe.query::<()>(config.connection.lock().unwrap().as_mut_connection_like()) {
                eprintln!("Error logging to Redis: {}", e);
            }
        }
    }

    fn flush(&self) {}
}

/// Configuration for the Redis logger.
pub struct RedisLoggerConfig {
    connection: Mutex<Box<dyn RedisConnection>>,
    channels: Option<(Vec<String>, Box<dyn PubSubEncoder>)>,
    streams: Option<(Vec<String>, Box<dyn StreamEncoder>)>,
}

impl RedisLoggerConfig {
    /// Initiates the builder pattern for creating a `RedisLoggerConfig`.
    pub fn builder() -> RedisLoggerConfigBuilder {
        RedisLoggerConfigBuilder::new()
    }
}

/// Builder for `RedisLoggerConfig`.
pub struct RedisLoggerConfigBuilder {
    redis_conn: Option<Box<dyn RedisConnection>>,
    channels: Option<(Vec<String>, Box<dyn PubSubEncoder>)>,
    streams: Option<(Vec<String>, Box<dyn StreamEncoder>)>,
}

impl RedisLoggerConfigBuilder {
    fn new() -> Self {
        Self {
            redis_conn: None,
            channels: None,
            streams: None,
        }
    }

    /// Sets the Redis client for the configuration. Mandatory.
    pub fn with_connection(mut self, redis_conn: Box<dyn RedisConnection>) -> Self {
        self.redis_conn = Some(redis_conn);
        self
    }

    #[cfg(feature = "default_encoding")]
    /// Sets the channels and encoder for the configuration. Either this or `with_streams` is mandatory.
    pub fn with_pubsub(mut self, channels: Vec<String>, encoder: Option<Box<dyn PubSubEncoder>>) -> Self {
        self.channels = Some((channels, encoder.unwrap_or(DefaultPubSubEncoder::new())));
        self
    }

    #[cfg(feature = "default_encoding")]
    /// Sets the streams and encoder for the configuration. Either this or `with_pubsub` is mandatory.
    pub fn with_streams(mut self, streams: Vec<String>, encoder: Option<Box<dyn StreamEncoder>>) -> Self {
        self.streams = Some((streams, encoder.unwrap_or(DefaultStreamEncoder::new())));
        self
    }

    #[cfg(not(feature = "default_encoding"))]
    /// Sets the channels and encoder for the configuration. Either this or `with_streams` is mandatory.
    pub fn with_pubsub(mut self, channels: Vec<String>, encoder: Box<dyn PubSubEncoder>) -> Self {
        self.channels = Some((channels, encoder));
        self
    }

    #[cfg(not(feature = "default_encoding"))]
    /// Sets the streams and encoder for the configuration. Either this or `with_pubsub` is mandatory.
    pub fn with_streams(mut self, streams: Vec<String>, encoder: Box<dyn StreamEncoder>) -> Self {
        self.streams = Some((streams, encoder));
        self
    }

    /// Builds the `RedisLoggerConfig` from the builder.
    ///
    /// # Returns
    ///
    /// A `RedisLoggerConfig` if successful or an error of type RedisLoggerConfigError.
    pub fn build(self) -> Result<RedisLoggerConfig, RedisLoggerConfigError> {
        let conn = self.redis_conn.ok_or(RedisLoggerConfigError::ClientNotSet)?;
        if self.channels.is_none() && self.streams.is_none() {
            return Err(RedisLoggerConfigError::ChannelNotSet);
        };
        if let Some((channels, _)) = &self.channels {
            if channels.is_empty() {
                return Err(RedisLoggerConfigError::ChannelNotSet);
            }
        }
        if let Some((streams, _)) = &self.streams {
            if streams.is_empty() {
                return Err(RedisLoggerConfigError::ChannelNotSet);
            }
        }

        Ok(RedisLoggerConfig {
            connection: Mutex::new(conn),
            channels: self.channels,
            streams: self.streams,
        })
    }
}
