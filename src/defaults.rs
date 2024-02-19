//! # Defaults Module
//!
//! This module provides default implementations for the `PubSubEncoder` and `StreamEncoder` traits.
//!
//! ## `DefaultPubSubEncoder`
//!
//! `DefaultPubSubEncoder` is a default implementation of the `PubSubEncoder` trait.
//! It encodes a `log::Record` into a JSON object, where each field in the `Record` becomes a key-value pair in the JSON object.
//! The JSON object is then converted into a byte vector.
//!
//! ## `DefaultStreamEncoder`
//!
//! `DefaultStreamEncoder` is a default implementation of the `StreamEncoder` trait.
//! It encodes a `log::Record` into a vector of tuples, where each tuple contains a field name from the `Record` and the
//! corresponding value as a byte vector. If a field in the `Record` is `None`, the byte vector is empty.
//!
//! ## Usage
//!
//! You can use these default encoders when you don't need to customize the encoding process.
//! If you need to customize the encoding, you can implement the `PubSubEncoder` and `StreamEncoder` traits yourself.

use serializable_log_record::SerializableLogRecord;

use super::{PubSubEncoder, Record, StreamEncoder};

/// Default implementation of the `PubSubEncoder` trait converting the incoming `log::Record` into a JSON object.
#[derive(Debug)]
#[non_exhaustive]
pub struct DefaultPubSubEncoder {}

impl DefaultPubSubEncoder {
    pub const fn new() -> Self {
        Self {}
    }
}

impl PubSubEncoder for DefaultPubSubEncoder {
    fn encode(&self, record: &Record) -> Vec<u8> {
        let ser_record = SerializableLogRecord::from(record);
        serde_json::to_string(&ser_record).unwrap().into_bytes()
    }
}

/// Default implementation of the `StreamEncoder` trait converting the incoming `log::Record` into a vector of tuples of field name and bytes.
#[derive(Debug)]
#[non_exhaustive]
pub struct DefaultStreamEncoder {}

impl DefaultStreamEncoder {
    pub const fn new() -> Self {
        Self {}
    }
}

impl StreamEncoder for DefaultStreamEncoder {
    fn encode(&self, record: &Record) -> Vec<(String, Vec<u8>)> {
        let ser_record = SerializableLogRecord::from(record);
        serde_json::to_value(&ser_record)
            .unwrap_or_else(|_| serde_json::json!({}))
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_owned().into_bytes()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::Level;

    #[test]
    fn test_default_pubsub_encoder_encode() {
        let encoder = DefaultPubSubEncoder::new();
        let record = Record::builder()
            .level(Level::Info)
            .args(format_args!("Test message"))
            .target("my_target")
            .module_path(Some("my_module"))
            .file(Some("my_file.rs"))
            .line(Some(42))
            .build();

        let expected = r#"{"level":"INFO","args":"Test message","target":"my_target","module_path":"my_module","file":"my_file.rs","line":42}"#;
        let expected_bytes = expected.as_bytes().to_vec();
        assert_eq!(encoder.encode(&record), expected_bytes);
    }

    #[test]
    fn test_default_stream_encoder_encode() {
        let encoder = DefaultStreamEncoder::new();
        let record = Record::builder()
            .level(Level::Error)
            .args(format_args!("Error message"))
            .target("my_target")
            .module_path(None)
            .file(Some("my_file.rs"))
            .line(None)
            .build();

        let expected = vec![
            ("args".to_owned(), b"Error message".to_vec()),
            ("file".to_owned(), b"my_file.rs".to_vec()),
            ("level".to_owned(), b"ERROR".to_vec()),
            ("line".to_owned(), b"".to_vec()),
            ("module_path".to_owned(), b"".to_vec()),
            ("target".to_owned(), b"my_target".to_vec()),
        ];

        assert_eq!(encoder.encode(&record), expected);
    }
}
