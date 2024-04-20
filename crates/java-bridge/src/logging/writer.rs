use crate::node::helpers::napi_error::MapToNapiError;
use app_state::{stateful, AppStateTrait, MutAppState, MutAppStateLock};
use log::{Level, Record};
use napi::threadsafe_function::ErrorStrategy;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi::Status;
use std::fmt::{Debug, Formatter};
use std::io;
use std::io::ErrorKind;
use std::ops::Not;

pub type LogFn = ThreadsafeFunction<String, ErrorStrategy::Fatal>;

#[derive(Eq, PartialEq, Copy, Clone)]
enum WriterType {
    Out,
    Err,
}

#[derive(Default)]
struct NodeWriterData {
    out: Option<LogFn>,
    err: Option<LogFn>,
}

impl Debug for NodeWriterData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeWriterData")
            .field("out", &"LogFn")
            .field("err", &"LogFn")
            .finish()
    }
}

#[derive(Debug)]
pub struct NodeWriter<'a> {
    record: &'a Record<'a>,
    out_buffer: Vec<u8>,
    err_buffer: Vec<u8>,
}

impl io::Write for NodeWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.record.level() >= Level::Info {
            self.out_buffer.append(&mut buf.to_vec());
        } else {
            self.err_buffer.append(&mut buf.to_vec());
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        NodeWriter::write(WriterType::Out, &mut self.out_buffer)?;
        NodeWriter::write(WriterType::Err, &mut self.err_buffer)
    }
}

impl log4rs::encode::Write for NodeWriter<'_> {}

impl<'a> NodeWriter<'a> {
    pub fn new(record: &'a Record<'a>) -> Self {
        Self {
            record,
            out_buffer: Vec::new(),
            err_buffer: Vec::new(),
        }
    }

    #[stateful(init(writers))]
    pub fn set_callbacks(
        out: Option<LogFn>,
        err: Option<LogFn>,
        mut writers: MutAppStateLock<NodeWriterData>,
    ) {
        writers.out = out;
        writers.err = err;
    }

    fn get_writer<'b>(
        writer_type: WriterType,
        writers: &'b MutAppStateLock<NodeWriterData>,
    ) -> Option<&'b LogFn> {
        match writer_type {
            WriterType::Out => writers.out.as_ref().or(writers.err.as_ref()),
            WriterType::Err => writers.err.as_ref().or(writers.out.as_ref()),
        }
    }

    #[stateful(init(writers))]
    fn write(
        writer_type: WriterType,
        buf: &mut Vec<u8>,
        writers: MutAppStateLock<NodeWriterData>,
    ) -> io::Result<()> {
        let Some(writer) = NodeWriter::get_writer(writer_type, &writers) else {
            return Ok(());
        };

        if let Some(data) = NodeWriter::convert(buf)? {
            NodeWriter::check_status(writer.call(data, ThreadsafeFunctionCallMode::NonBlocking))?;
        }

        Ok(())
    }

    fn convert(data: &mut Vec<u8>) -> io::Result<Option<String>> {
        data.is_empty()
            .not()
            .then(|| {
                String::from_utf8(
                    data.drain(
                        0..data[data.len() - 1]
                            .eq(&b'\n')
                            .then(|| data.len() - 1)
                            .unwrap_or(data.len()),
                    )
                    .collect(),
                )
                .map_napi_err(None)
            })
            .map_or(Ok(None), |v| v.map(Some))
            .map_err(|e| io::Error::new(ErrorKind::Other, e))
    }

    fn check_status(status: Status) -> io::Result<()> {
        if status == Status::Ok {
            Ok(())
        } else {
            Err(io::Error::new(
                ErrorKind::Other,
                format!("Failed to call JS function. Error: {}", status),
            ))
        }
    }
}
