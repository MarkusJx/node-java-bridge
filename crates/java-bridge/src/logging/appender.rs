use crate::logging::writer::NodeWriter;
use log::Record;
use log4rs::append::Append;
use log4rs::config::{Deserialize, Deserializers};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::encode::{Encode, EncoderConfig};
use std::io::Write;

#[derive(Debug)]
pub struct NodeAppender {
    encoder: Box<dyn Encode>,
}

impl Append for NodeAppender {
    fn append(&self, record: &Record) -> anyhow::Result<()> {
        let mut writer = NodeWriter::new(record);
        self.encoder.encode(&mut writer, record)?;
        writer.flush().map_err(Into::into)
    }

    fn flush(&self) {}
}

impl NodeAppender {
    pub fn builder() -> NodeAppenderBuilder {
        NodeAppenderBuilder { encoder: None }
    }
}

pub struct NodeAppenderBuilder {
    encoder: Option<Box<dyn Encode>>,
}

impl NodeAppenderBuilder {
    pub fn encoder(mut self, encoder: Box<dyn Encode>) -> Self {
        self.encoder = Some(encoder);
        self
    }

    pub fn build(self) -> NodeAppender {
        NodeAppender {
            encoder: self
                .encoder
                .unwrap_or_else(|| Box::new(PatternEncoder::default())),
        }
    }
}

#[derive(serde::Deserialize)]
pub struct NodeAppenderConfig {
    encoder: Option<EncoderConfig>,
}

pub struct NodeAppenderSerializer;

impl Deserialize for NodeAppenderSerializer {
    type Trait = dyn Append;
    type Config = NodeAppenderConfig;

    fn deserialize(
        &self,
        config: Self::Config,
        deserializers: &Deserializers,
    ) -> anyhow::Result<Box<Self::Trait>> {
        let mut builder = NodeAppender::builder();
        if let Some(encoder) = config.encoder {
            builder = builder.encoder(deserializers.deserialize(&encoder.kind, encoder.config)?);
        }

        Ok(Box::new(builder.build()))
    }
}
