pub mod schemas;

use anyhow::{Context, Result};
use nng::{Protocol, Socket};

pub fn log(msg: impl Into<String>) {
    let buf = serialize_log(&msg.into());

    let socket = Socket::new(Protocol::Push0).expect("Failed to create a new socket");
    socket
        .dial("tcp://127.0.0.1:62000")
        .expect("Failed to connect to the server");
    socket.send(&buf).expect("Failed to send message");
    socket.close();

    println!("Log sent successfully");
}

fn serialize_log(msg: &str) -> Vec<u8> {
    let mut builder = flatbuffers::FlatBufferBuilder::with_capacity(1024);
    let time: i64 = chrono::Utc::now().timestamp_millis() as i64;
    let msg_string = builder.create_string(msg);
    let log_entry = schemas::log::log::Log::create(
        &mut builder,
        &schemas::log::log::LogArgs {
            msg: Some(msg_string),
            ts: time,
        },
    );

    builder.finish(log_entry, None);
    builder.finished_data().to_vec()
}

pub fn deserialize_log(buf: &[u8]) -> Result<schemas::log::log::Log> {
    flatbuffers::root::<schemas::log::log::Log>(buf).context("Failed to deserialize Log")
}
