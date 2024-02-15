use super::*;
use mockall::mock;

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
}

const DUMMY_PUBSUB_ENCODER: DummyPubSubEncoder = DummyPubSubEncoder { __private: PhantomData };
const DUMMY_STREAM_ENCODER: DummyStreamEncoder = DummyStreamEncoder { __private: PhantomData };

#[test]
fn test_build_only_streams() {
    let streams = vec!["stream1".into(), "stream2".into()];
    let mock_conn = MockRedisConnection::new();

    let config = RedisLoggerConfigBuilder::build_with_streams(mock_conn, streams, DUMMY_STREAM_ENCODER);

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
    let mock_conn = MockRedisConnection::new();

    let config = RedisLoggerConfigBuilder::build_with_pubsub(mock_conn, channels, DUMMY_PUBSUB_ENCODER);

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
    let mock_conn = MockRedisConnection::new();

    let config = RedisLoggerConfigBuilder::build_with_pubsub_and_streams(
        mock_conn,
        channels,
        DUMMY_PUBSUB_ENCODER,
        streams,
        DUMMY_STREAM_ENCODER,
    );

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
#[should_panic]
fn test_build_only_pubsub_but_no_channels() {
    let channels = vec![];
    let mock_conn = MockRedisConnection::new();

    RedisLoggerConfigBuilder::build_with_pubsub(mock_conn, channels, DUMMY_PUBSUB_ENCODER);
}

#[test]
#[should_panic]
fn test_build_only_streams_but_no_channels() {
    let channels = vec![];
    let mock_conn = MockRedisConnection::new();

    RedisLoggerConfigBuilder::build_with_streams(mock_conn, channels, DUMMY_STREAM_ENCODER);
}

#[test]
#[should_panic]
fn test_build_pubsub_and_streams_but_no_channels() {
    let streams = vec![];
    let channels = vec![];
    let mock_conn = MockRedisConnection::new();

    RedisLoggerConfigBuilder::build_with_pubsub_and_streams(
        mock_conn,
        channels,
        DUMMY_PUBSUB_ENCODER,
        streams,
        DUMMY_STREAM_ENCODER,
    );
}

#[cfg(feature = "default_encoding")]
#[test]
fn test_build_only_streams_default() {
    use std::any::{Any, TypeId};

    let streams = vec!["stream1".into(), "stream2".into()];
    let mock_conn = MockRedisConnection::new();

    let config = RedisLoggerConfigBuilder::build_with_streams_default(mock_conn, streams);

    assert!(config.channels.is_none());
    assert!(config.streams.is_some());
    assert_eq!(
        config.streams.as_ref().unwrap().0,
        vec!["stream1".to_string(), "stream2".to_string()]
    );
    assert_eq!(
        config.streams.as_ref().unwrap().1.type_id(),
        TypeId::of::<DefaultStreamEncoder>()
    );
    assert!(config.connection.lock().is_ok());
}

#[cfg(feature = "default_encoding")]
#[test]
fn test_build_only_pubsub_default() {
    use std::any::{Any, TypeId};

    let channels = vec!["channel1".into(), "channel2".into()];
    let mock_conn = MockRedisConnection::new();

    let config = RedisLoggerConfigBuilder::build_with_pubsub_default(mock_conn, channels);

    assert!(config.channels.is_some());
    assert!(config.streams.is_none());
    assert_eq!(
        config.channels.as_ref().unwrap().0,
        vec!["channel1".to_string(), "channel2".to_string()]
    );
    assert_eq!(
        config.channels.as_ref().unwrap().1.type_id(),
        TypeId::of::<DefaultPubSubEncoder>()
    );
    assert!(config.connection.lock().is_ok());
}

#[cfg(feature = "default_encoding")]
#[test]
fn test_build_pubsub_and_streams_default() {
    use std::any::{Any, TypeId};

    let channels = vec!["channel1".into(), "channel2".into()];
    let streams = vec!["stream1".into(), "stream2".into()];
    let mock_conn = MockRedisConnection::new();

    let config = RedisLoggerConfigBuilder::build_with_pubsub_and_streams_default(mock_conn, channels, streams);

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
    assert_eq!(
        config.channels.as_ref().unwrap().1.type_id(),
        TypeId::of::<DefaultPubSubEncoder>()
    );
    assert_eq!(
        config.streams.as_ref().unwrap().1.type_id(),
        TypeId::of::<DefaultStreamEncoder>()
    );
    assert!(config.connection.lock().is_ok());
}

#[cfg(feature = "default_encoding")]
#[test]
#[should_panic]
fn test_build_only_pubsub_but_no_channels_default() {
    let channels = vec![];
    let mock_conn = MockRedisConnection::new();

    RedisLoggerConfigBuilder::build_with_pubsub_default(mock_conn, channels);
}

#[cfg(feature = "default_encoding")]
#[test]
#[should_panic]
fn test_build_only_streams_but_no_channels_default() {
    let channels = vec![];
    let mock_conn = MockRedisConnection::new();

    RedisLoggerConfigBuilder::build_with_streams_default(mock_conn, channels);
}

#[cfg(feature = "default_encoding")]
#[test]
#[should_panic]
fn test_build_pubsub_and_streams_but_no_channels_default() {
    let streams = vec![];
    let channels = vec![];
    let mock_conn = MockRedisConnection::new();

    RedisLoggerConfigBuilder::build_with_pubsub_and_streams_default(mock_conn, channels, streams);
}
