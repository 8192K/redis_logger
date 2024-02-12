use log::SetLoggerError;
use redis::RedisError;

#[derive(Debug, thiserror::Error)]
pub enum RedisLoggerConfigError {
    #[error("Redis client not set")]
    ClientNotSet(),
    #[error("Channels not set. Set at least one pub/sub channel and/or one stream name.")]
    ChannelNotSet(),
    #[error("Error handling Redis: {0}")]
    RedisError(#[from] RedisError),
    #[error("Error initializing logger: {0}")]
    SetLoggerError(#[from] SetLoggerError),
}
