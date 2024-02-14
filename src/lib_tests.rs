use super::*;
use mockall::{mock, predicate::*};

// Mock RedisConnection
mock! {
    pub RedisConnection {}
    impl ConnectionLike for RedisConnection {
        fn req_packed_commands(&mut self, cmd: &[u8], a: usize, b: usize) -> Result<Vec<redis::Value>, redis::RedisError>;
        fn req_packed_command(&mut self, cmd: &[u8]) -> Result<redis::Value, redis::RedisError>;
        fn get_db(&self) -> i64;
        fn check_connection(&mut self) -> bool;
        fn is_open(&self) -> bool;
    }
    unsafe impl Sync for RedisConnection {}
    unsafe impl Send for RedisConnection {}
    impl RedisConnection for RedisConnection {
        fn as_mut_connection_like(&mut self) -> &mut dyn ConnectionLike;
    }
}

// Mock PubSubEncoder
mock! {
    pub PubSubEncoder {}
    impl PubSubEncoder for PubSubEncoder {
        fn encode<'a>(&self, record: &Record<'a>) -> Vec<u8>;
        fn name(&self) -> &'static str;
    }
}

// Mock StreamEncoder
mock! {
    pub StreamEncoder {}
    impl StreamEncoder for StreamEncoder {
        fn encode<'a>(&self, record: &Record<'a>) -> Vec<(&'static str, Vec<u8>)>;
        fn name(&self) -> &'static str;
    }
}

fn to_pointer_address<T>(t: &T) -> usize
where
    T: ?Sized,
{
    t as *const T as *const () as *const u8 as usize
}

#[test]
fn test_with_connection() {
    let mock_conn = Box::new(MockRedisConnection::new());
    let address_conn = to_pointer_address(&*mock_conn);

    let builder = RedisLoggerConfigBuilder::new().with_connection(mock_conn);

    let conn = &builder.redis_conn;
    assert!(conn.is_some());

    let address_conn_builder = to_pointer_address(&**conn.as_ref().unwrap());
    assert_eq!(address_conn, address_conn_builder);
}

#[test]
fn test_with_pubsub() {
    let channels = vec!["channel1".into(), "channel2".into()];
    let mock_encoder = Box::new(MockPubSubEncoder::new());

    let address_channels_string0 = to_pointer_address(&channels[0]);
    let address_channels_string1 = to_pointer_address(&channels[1]);
    let address_encoder = to_pointer_address(&*mock_encoder);

    #[cfg(feature = "default_encoding")]
    let builder = RedisLoggerConfigBuilder::new().with_pubsub(channels, Some(mock_encoder));

    #[cfg(not(feature = "default_encoding"))]
    let builder = RedisLoggerConfigBuilder::new().with_pubsub(channels, mock_encoder);

    let c = &builder.channels;
    assert!(c.is_some());

    let address_channels_string0_builder = to_pointer_address(&c.as_ref().unwrap().0[0]);
    assert_eq!(address_channels_string0_builder, address_channels_string0);

    let address_channels_string1_builder = to_pointer_address(&c.as_ref().unwrap().0[1]);
    assert_eq!(address_channels_string1_builder, address_channels_string1);

    let address_encoder_builder = to_pointer_address(&*c.as_ref().unwrap().1);
    assert_eq!(address_encoder_builder, address_encoder);
}

#[test]
fn test_with_streams() {
    let streams = vec!["stream1".into(), "stream2".into()];
    let mock_encoder = Box::new(MockStreamEncoder::new());

    let address_streams_string0 = to_pointer_address(&streams[0]);
    let address_streams_string1 = to_pointer_address(&streams[1]);
    let address_encoder = to_pointer_address(&*mock_encoder);

    #[cfg(feature = "default_encoding")]
    let builder = RedisLoggerConfigBuilder::new().with_streams(streams, Some(mock_encoder));

    #[cfg(not(feature = "default_encoding"))]
    let builder = RedisLoggerConfigBuilder::new().with_streams(streams, mock_encoder);

    let c = &builder.streams;
    assert!(c.is_some());

    let address_streams_string0_builder = to_pointer_address(&c.as_ref().unwrap().0[0]);
    assert_eq!(address_streams_string0_builder, address_streams_string0);

    let address_streams_string1_builder = to_pointer_address(&c.as_ref().unwrap().0[1]);
    assert_eq!(address_streams_string1_builder, address_streams_string1);

    let address_encoder_builder = to_pointer_address(&*c.as_ref().unwrap().1);
    assert_eq!(address_encoder_builder, address_encoder);
}

#[test]
fn test_build_only_streams() {
    let streams = vec!["stream1".into(), "stream2".into()];
    let mock_encoder = Box::new(MockStreamEncoder::new());
    let mock_conn = Box::new(MockRedisConnection::new());

    #[cfg(feature = "default_encoding")]
    let config = RedisLoggerConfigBuilder::new()
        .with_connection(mock_conn)
        .with_streams(streams, Some(mock_encoder))
        .build();

    #[cfg(not(feature = "default_encoding"))]
    let config = RedisLoggerConfigBuilder::new()
        .with_connection(mock_conn)
        .with_streams(streams, mock_encoder)
        .build();

    assert!(config.is_ok());
    let config = config.unwrap();

    assert!(config.channels.is_none());
    assert!(config.streams.is_some());
    assert_eq!(
        config.streams.as_ref().unwrap().0,
        vec!["stream1".to_string(), "stream2".to_string()]
    );
    assert!(config.connection.lock().is_ok());
}

#[test]
fn test_build_only_pubsub() {
    let channels = vec!["channel1".into(), "channel2".into()];
    let mock_encoder = Box::new(MockPubSubEncoder::new());
    let mock_conn = Box::new(MockRedisConnection::new());

    #[cfg(feature = "default_encoding")]
    let config = RedisLoggerConfigBuilder::new()
        .with_connection(mock_conn)
        .with_pubsub(channels, Some(mock_encoder))
        .build();

    #[cfg(not(feature = "default_encoding"))]
    let config = RedisLoggerConfigBuilder::new()
        .with_connection(mock_conn)
        .with_pubsub(channels, mock_encoder)
        .build();

    assert!(config.is_ok());
    let config = config.unwrap();

    assert!(config.channels.is_some());
    assert!(config.streams.is_none());
    assert_eq!(
        config.channels.as_ref().unwrap().0,
        vec!["channel1".to_string(), "channel2".to_string()]
    );
    assert!(config.connection.lock().is_ok());
}

#[test]
fn test_build_pubsub_and_streams() {
    let channels = vec!["channel1".into(), "channel2".into()];
    let streams = vec!["stream1".into(), "stream2".into()];
    let mock_pubsub_encoder = Box::new(MockPubSubEncoder::new());
    let mock_stream_encoder = Box::new(MockStreamEncoder::new());
    let mock_conn = Box::new(MockRedisConnection::new());

    #[cfg(feature = "default_encoding")]
    let config = RedisLoggerConfigBuilder::new()
        .with_connection(mock_conn)
        .with_pubsub(channels, Some(mock_pubsub_encoder))
        .with_streams(streams, Some(mock_stream_encoder))
        .build();

    #[cfg(not(feature = "default_encoding"))]
    let config = RedisLoggerConfigBuilder::new()
        .with_connection(mock_conn)
        .with_pubsub(channels, mock_pubsub_encoder)
        .with_streams(streams, mock_stream_encoder)
        .build();

    assert!(config.is_ok());
    let config = config.unwrap();

    assert!(config.channels.is_some());
    assert!(config.streams.is_some());
    assert_eq!(
        config.channels.as_ref().unwrap().0,
        vec!["channel1".to_string(), "channel2".to_string()]
    );
    assert_eq!(
        config.streams.as_ref().unwrap().0,
        vec!["stream1".to_string(), "stream2".to_string()]
    );
    assert!(config.connection.lock().is_ok());
}

#[test]
fn test_build_no_connection_and_no_pubsub_or_streams() {
    let builder = RedisLoggerConfigBuilder::new();
    let config = builder.build();
    assert!(config.is_err());
    assert_eq!(config.err().unwrap(), RedisLoggerConfigError::ClientNotSet);
}

#[test]
fn test_build_missing_connection() {
    let channels = vec!["channel1".into(), "channel2".into()];
    let mock_pubsub_encoder = Box::new(MockPubSubEncoder::new());

    #[cfg(feature = "default_encoding")]
    let config = RedisLoggerConfigBuilder::new()
        .with_pubsub(channels, Some(mock_pubsub_encoder))
        .build();

    #[cfg(not(feature = "default_encoding"))]
    let config = RedisLoggerConfigBuilder::new()
        .with_pubsub(channels, mock_pubsub_encoder)
        .build();

    assert!(config.is_err());
    assert_eq!(config.err().unwrap(), RedisLoggerConfigError::ClientNotSet);
}

#[test]
fn test_build_only_pubsub_but_no_channels() {
    let channels = vec![];
    let mock_pubsub_encoder = Box::new(MockPubSubEncoder::new());
    let mock_conn = Box::new(MockRedisConnection::new());

    #[cfg(feature = "default_encoding")]
    let config = RedisLoggerConfigBuilder::new()
        .with_connection(mock_conn)
        .with_pubsub(channels, Some(mock_pubsub_encoder))
        .build();

    #[cfg(not(feature = "default_encoding"))]
    let config = RedisLoggerConfigBuilder::new()
        .with_connection(mock_conn)
        .with_pubsub(channels, mock_pubsub_encoder)
        .build();

    assert!(config.is_err());
    assert_eq!(config.err().unwrap(), RedisLoggerConfigError::ChannelNotSet);
}

#[test]
fn test_build_only_streams_but_no_channels() {
    let streams = vec![];
    let mock_stream_encoder = Box::new(MockStreamEncoder::new());
    let mock_conn = Box::new(MockRedisConnection::new());

    #[cfg(feature = "default_encoding")]
    let config = RedisLoggerConfigBuilder::new()
        .with_connection(mock_conn)
        .with_streams(streams, Some(mock_stream_encoder))
        .build();

    #[cfg(not(feature = "default_encoding"))]
    let config = RedisLoggerConfigBuilder::new()
        .with_connection(mock_conn)
        .with_streams(streams, mock_stream_encoder)
        .build();

    assert!(config.is_err());
    assert_eq!(config.err().unwrap(), RedisLoggerConfigError::ChannelNotSet);
}

#[test]
fn test_build_pubsub_and_streams_but_no_channels() {
    let streams = vec![];
    let channels = vec![];
    let mock_pubsub_encoder = Box::new(MockPubSubEncoder::new());
    let mock_stream_encoder = Box::new(MockStreamEncoder::new());
    let mock_conn = Box::new(MockRedisConnection::new());

    #[cfg(feature = "default_encoding")]
    let config = RedisLoggerConfigBuilder::new()
        .with_connection(mock_conn)
        .with_pubsub(channels, Some(mock_pubsub_encoder))
        .with_streams(streams, Some(mock_stream_encoder))
        .build();

    #[cfg(not(feature = "default_encoding"))]
    let config = RedisLoggerConfigBuilder::new()
        .with_connection(mock_conn)
        .with_pubsub(channels, mock_pubsub_encoder)
        .with_streams(streams, mock_stream_encoder)
        .build();

    assert!(config.is_err());
    assert_eq!(config.err().unwrap(), RedisLoggerConfigError::ChannelNotSet);
}

#[cfg(feature = "default_encoding")]
#[test]
fn test_build_pubsub_and_streams_with_defaults() {
    let channels = vec!["channel1".into(), "channel2".into()];
    let streams = vec!["stream1".into(), "stream2".into()];
    let mock_conn = Box::new(MockRedisConnection::new());

    let config = RedisLoggerConfigBuilder::new()
        .with_connection(mock_conn)
        .with_pubsub(channels, None)
        .with_streams(streams, None)
        .build();

    assert!(config.is_ok());
    let config = config.unwrap();

    assert!(config.channels.is_some());
    assert!(config.channels.is_some());

    let c = config.channels.unwrap();
    let s = config.streams.unwrap();

    assert_eq!(c.0, vec!["channel1".to_string(), "channel2".to_string()]);
    assert_eq!(s.0, vec!["stream1".to_string(), "stream2".to_string()]);

    assert_eq!(c.1.name(), "DefaultPubSubEncoder");
    assert_eq!(s.1.name(), "DefaultStreamEncoder");
}
