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
//! corresponding value as a byte vector. If a field in the `Record` is `None`, it uses a default value.
//!
//! ## Usage
//!
//! You can use these default encoders when you don't need to customize the encoding process.
//! If you need to customize the encoding, you can implement the `PubSubEncoder` and `StreamEncoder` traits yourself.

use super::{PubSubEncoder, Record, StreamEncoder};

/// Default implementation of the `PubSubEncoder` trait converting the incoming `log::Record` into a JSON object.
pub struct DefaultPubSubEncoder;

impl DefaultPubSubEncoder {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

impl PubSubEncoder for DefaultPubSubEncoder {
    fn encode<'a>(&self, record: &Record<'a>) -> Vec<u8> {
        let json = serde_json::json!({
         "level": record.level().as_str(),
         "args": record.args().to_string(),
         "module_path": record.module_path().map(str::to_owned),
         "target": record.target().to_owned(),
         "file": record.file().map(str::to_owned),
         "line": record.line()
        });
        json.to_string().into_bytes()
    }

    #[cfg(test)]
    fn name(&self) -> &'static str {
        "DefaultPubSubEncoder"
    }
}

/// Default implementation of the `StreamEncoder` trait converting the incoming `log::Record` into a vector of tuples.
pub struct DefaultStreamEncoder;

impl DefaultStreamEncoder {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

impl StreamEncoder for DefaultStreamEncoder {
    fn encode<'a>(&self, record: &Record<'a>) -> Vec<(&'static str, Vec<u8>)> {
        vec![
            ("level", record.level().as_str().to_owned().into_bytes()),
            ("args", record.args().to_string().into_bytes()),
            ("module_path", record.module_path().unwrap_or("null").to_owned().into_bytes()),
            ("target", record.target().to_owned().into_bytes()),
            ("file", record.file().unwrap_or("null").to_owned().into_bytes()),
            ("line", record.line().unwrap_or(0).to_string().into_bytes()),
        ]
    }

    #[cfg(test)]
    fn name(&self) -> &'static str {
        "DefaultStreamEncoder"
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
            .module_path(Some("my_module"))
            .target("my_target")
            .file(Some("my_file.rs"))
            .line(Some(42))
            .build();

        let expected = r#"{"args":"Test message","file":"my_file.rs","level":"INFO","line":42,"module_path":"my_module","target":"my_target"}"#;
        let expected_bytes = expected.as_bytes().to_vec();
        assert_eq!(encoder.encode(&record), expected_bytes);
    }

    #[test]
    fn test_default_stream_encoder_encode() {
        let encoder = DefaultStreamEncoder::new();
        let record = Record::builder()
            .level(Level::Error)
            .args(format_args!("Error message"))
            .module_path(None)
            .target("my_target")
            .file(Some("my_file.rs"))
            .line(None)
            .build();

        let expected = vec![
            ("level", b"ERROR".to_vec()),
            ("args", b"Error message".to_vec()),
            ("module_path", b"null".to_vec()),
            ("target", b"my_target".to_vec()),
            ("file", b"my_file.rs".to_vec()),
            ("line", b"0".to_vec()),
        ];

        assert_eq!(encoder.encode(&record), expected);
    }
}
