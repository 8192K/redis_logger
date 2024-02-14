//! # Error Module
//!
//! This module defines the `RedisLoggerConfigError` enum, which represents the possible errors that can occur in the `RedisLogger` configuration.
//!
//! ## `RedisLoggerConfigError`
//!
//! `RedisLoggerConfigError` is an enum with variants for each possible error:
//!
//! - `ClientNotSet`: This error indicates that the Redis client is not set.
//! - `ChannelNotSet`: This error indicates that the channels are not set. At least one pub/sub channel and/or one stream name should be set.
//! - `RedisError`: This error indicates that an error occurred while handling Redis. It wraps an error from the `redis` crate.
//! - `SetLoggerError`: This error indicates that an error occurred while initializing the logger. It wraps an error from the `log` crate.
//!
//! Each variant carries the necessary information to describe the error. For `RedisError` and `SetLoggerError`,
//! this includes the original error from the `redis` or `log` crate.
//!
//! ## Usage
//!
//! These errors can be used in `Result` return types to indicate that an operation related to the `RedisLogger` configuration failed.
//! The caller can then handle these errors appropriately.

/// Represents the possible errors that can occur in the `RedisLogger` configuration.
#[derive(Debug, thiserror::Error)]
pub enum RedisLoggerConfigError {
    /// Error indicating that the Redis client is not set.
    #[error("Redis client not set")]
    ClientNotSet,

    /// Error indicating that the channels are not set.
    #[error("Channels not set. Set at least one pub/sub channel and/or one stream name.")]
    ChannelNotSet,

    /// Error indicating an error occurred while handling Redis.
    #[error("Error handling Redis: {0}")]
    RedisError(#[from] redis::RedisError),

    /// Error indicating an error occurred while initializing the logger.
    #[error("Error initializing logger: {0}")]
    SetLoggerError(#[from] log::SetLoggerError),
}

impl PartialEq for RedisLoggerConfigError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::ClientNotSet, Self::ClientNotSet)
                | (Self::ChannelNotSet, Self::ChannelNotSet)
                | (Self::RedisError(_), Self::RedisError(_))
                | (Self::SetLoggerError(_), Self::SetLoggerError(_))
        )
    }
}
