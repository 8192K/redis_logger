use super::*;

const DUMMY_PUBSUB_ENCODER: DummyPubSubEncoder = DummyPubSubEncoder {};
const DUMMY_STREAM_ENCODER: DummyStreamEncoder = DummyStreamEncoder {};

#[test]
fn test_build_only_streams() {
    let streams = vec!["stream1".into(), "stream2".into()];

    let config = RedisLoggerConfigBuilder::with_streams(String::new(), streams, DUMMY_STREAM_ENCODER);

    assert!(config.channels.is_none());
    assert!(config.streams.is_some());
    assert_eq!(
        config.streams.as_ref().unwrap().0,
        vec!["stream1".to_string(), "stream2".to_string()]
    );
}

#[test]
fn test_build_only_pubsub() {
    let channels = vec!["channel1".into(), "channel2".into()];

    let config = RedisLoggerConfigBuilder::with_pubsub(String::new(), channels, DUMMY_PUBSUB_ENCODER);

    assert!(config.channels.is_some());
    assert!(config.streams.is_none());
    assert_eq!(
        config.channels.as_ref().unwrap().0,
        vec!["channel1".to_string(), "channel2".to_string()]
    );
}

#[test]
fn test_build_pubsub_and_streams() {
    let channels = vec!["channel1".into(), "channel2".into()];
    let streams = vec!["stream1".into(), "stream2".into()];

    let config = RedisLoggerConfigBuilder::with_pubsub_and_streams(
        String::new(),
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
}

#[test]
#[should_panic]
fn test_build_only_pubsub_but_no_channels() {
    let channels = vec![];
    RedisLoggerConfigBuilder::with_pubsub(String::new(), channels, DUMMY_PUBSUB_ENCODER);
}

#[test]
#[should_panic]
fn test_build_only_streams_but_no_channels() {
    let channels = vec![];
    RedisLoggerConfigBuilder::with_streams(String::new(), channels, DUMMY_STREAM_ENCODER);
}

#[test]
#[should_panic]
fn test_build_pubsub_and_streams_but_no_channels() {
    let streams = vec![];
    let channels = vec![];

    RedisLoggerConfigBuilder::with_pubsub_and_streams(
        String::new(),
        channels,
        DUMMY_PUBSUB_ENCODER,
        streams,
        DUMMY_STREAM_ENCODER,
    );
}

#[cfg(feature = "default_encoders")]
#[test]
fn test_build_only_streams_default() {
    use std::any::{Any, TypeId};

    let streams = vec!["stream1".into(), "stream2".into()];

    let config = RedisLoggerConfigBuilder::with_streams_default(String::new(), streams);

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
}

#[cfg(feature = "default_encoders")]
#[test]
fn test_build_only_pubsub_default() {
    use std::any::{Any, TypeId};

    let channels = vec!["channel1".into(), "channel2".into()];

    let config = RedisLoggerConfigBuilder::with_pubsub_default(String::new(), channels);

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
}

#[cfg(feature = "default_encoders")]
#[test]
fn test_build_pubsub_and_streams_default() {
    use std::any::{Any, TypeId};

    let channels = vec!["channel1".into(), "channel2".into()];
    let streams = vec!["stream1".into(), "stream2".into()];

    let config = RedisLoggerConfigBuilder::with_pubsub_and_streams_default(String::new(), channels, streams);

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
}

#[cfg(feature = "default_encoders")]
#[test]
#[should_panic]
fn test_build_only_pubsub_but_no_channels_default() {
    let channels = vec![];
    RedisLoggerConfigBuilder::with_pubsub_default(String::new(), channels);
}

#[cfg(feature = "default_encoders")]
#[test]
#[should_panic]
fn test_build_only_streams_but_no_channels_default() {
    let channels = vec![];
    RedisLoggerConfigBuilder::with_streams_default(String::new(), channels);
}

#[cfg(feature = "default_encoders")]
#[test]
#[should_panic]
fn test_build_pubsub_and_streams_but_no_channels_default() {
    let streams = vec![];
    let channels = vec![];
    RedisLoggerConfigBuilder::with_pubsub_and_streams_default(String::new(), channels, streams);
}
