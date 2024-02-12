use super::{PubSubEncoder, Record, StreamEncoder};

pub struct DefaultPubSubEncoder;

impl DefaultPubSubEncoder {
    pub fn new() -> Self {
        Self {}
    }
}

impl PubSubEncoder for DefaultPubSubEncoder {
    fn encode(&self, record: &Record) -> Vec<u8> {
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
}

pub struct DefaultStreamEncoder;

impl DefaultStreamEncoder {
    pub fn new() -> Self {
        Self {}
    }
}

impl StreamEncoder for DefaultStreamEncoder {
    fn encode(&self, record: &Record) -> Vec<(&str, Vec<u8>)> {
        vec![
            ("level", record.level().as_str().to_owned().into_bytes()),
            ("args", record.args().to_string().into_bytes()),
            ("module_path", record.module_path().unwrap_or("null").to_owned().into_bytes()),
            ("target", record.target().to_owned().into_bytes()),
            ("file", record.file().unwrap_or("null").to_owned().into_bytes()),
            ("line", record.line().unwrap_or(0).to_string().into_bytes()),
        ]
    }
}
