use std::sync::Mutex;

use log::{LevelFilter, Log, Metadata, Record};
use redis::{Client, Connection};

mod error;
pub use error::RedisLoggerConfigError;

#[cfg(feature = "default_encoding")]
mod defaults;
#[cfg(feature = "default_encoding")]
pub use defaults::*;

pub trait PubSubEncoder: Send + Sync {
    fn encode(&self, record: &Record) -> Vec<u8>;
}

pub trait StreamEncoder: Send + Sync {
    fn encode(&self, record: &Record) -> Vec<(&str, Vec<u8>)>;
}

pub struct RedisLogger {
    level: LevelFilter,
    config: RedisLoggerConfig,
}

impl RedisLogger {
    pub fn new(level: LevelFilter, config: RedisLoggerConfig) -> Box<Self> {
        Box::new(Self { level, config })
    }

    pub fn init(level: LevelFilter, config: RedisLoggerConfig) -> Result<(), RedisLoggerConfigError> {
        let redis_logger = RedisLogger::new(level, config);
        log::set_max_level(level);
        log::set_boxed_logger(redis_logger)?;
        Ok(())
    }
}

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
            if let Err(e) = pipe.query::<()>(&mut config.connection.lock().unwrap()) {
                eprintln!("Error logging to Redis: {}", e);
            }
        }
    }

    fn flush(&self) {}
}

pub struct RedisLoggerConfig {
    connection: Mutex<Connection>,
    channels: Option<(Vec<String>, Box<dyn PubSubEncoder>)>,
    streams: Option<(Vec<String>, Box<dyn StreamEncoder>)>,
}

impl RedisLoggerConfig {
    pub fn builder() -> RedisLoggerConfigBuilder {
        RedisLoggerConfigBuilder::new()
    }
}

pub struct RedisLoggerConfigBuilder {
    client: Option<Client>,
    channels: Option<(Vec<String>, Box<dyn PubSubEncoder>)>,
    streams: Option<(Vec<String>, Box<dyn StreamEncoder>)>,
}

impl RedisLoggerConfigBuilder {
    fn new() -> Self {
        Self {
            client: None,
            channels: None,
            streams: None,
        }
    }

    pub fn with_client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }

    #[cfg(feature = "default_encoding")]
    pub fn with_pubsub(mut self, channels: (Vec<String>, Option<Box<dyn PubSubEncoder>>)) -> Self {
        self.channels = Some((channels.0, channels.1.unwrap_or(Box::new(DefaultPubSubEncoder::new()))));
        self
    }

    #[cfg(feature = "default_encoding")]
    pub fn with_streams(mut self, streams: (Vec<String>, Option<Box<dyn StreamEncoder>>)) -> Self {
        self.streams = Some((streams.0, streams.1.unwrap_or(Box::new(DefaultStreamEncoder::new()))));
        self
    }

    #[cfg(not(feature = "default_encoding"))]
    pub fn with_pubsub(mut self, channels: (Vec<String>, Box<dyn PubSubEncoder>)) -> Self {
        self.channels = Some(channels);
        self
    }

    #[cfg(not(feature = "default_encoding"))]
    pub fn with_streams(mut self, streams: (Vec<String>, Box<dyn StreamEncoder>)) -> Self {
        self.streams = Some(streams);
        self
    }

    pub fn build(self) -> Result<RedisLoggerConfig, RedisLoggerConfigError> {
        let client = self.client.ok_or(RedisLoggerConfigError::ClientNotSet())?;
        if self.channels.is_none() && self.streams.is_none() {
            return Err(RedisLoggerConfigError::ChannelNotSet());
        };

        let connection = client.get_connection()?;
        Ok(RedisLoggerConfig {
            connection: Mutex::new(connection),
            channels: self.channels,
            streams: self.streams,
        })
    }
}
