use std::{
    ffi::OsString,
    process::{self, Stdio},
};

use clap::Parser;
use futures::{SinkExt, TryStreamExt};
use muxeo::{Frame, FrameKind, MAX};
use tokio::{io, process::Command};
use tokio_stream::StreamExt;
use tokio_util::{
    bytes::{BufMut, BytesMut},
    codec::{Encoder, FramedWrite},
    io::ReaderStream,
};

/// Multiplexer for standard error and standard output
#[derive(Parser)]
#[command(version)]
struct Cli {
    program: OsString,
    args: Vec<OsString>,
}

struct EoEncoder;

impl Encoder<Frame> for EoEncoder {
    type Error = io::Error;

    fn encode(&mut self, item: Frame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let (kind, bytes) = match item {
            Frame::Err(bytes) => (FrameKind::Err, bytes),
            Frame::Out(bytes) => (FrameKind::Out, bytes),
        };

        let bytes_len = bytes.len();

        // Don't send a string if it is longer than the other end will accept.
        if bytes_len > MAX {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Frame of length {} is too large.", bytes_len),
            ));
        }

        // Reserve space in the buffer.
        dst.reserve(1 + 4 + bytes_len);

        // Write the kind, length and string to the buffer.
        dst.put_u8(kind as u8);
        // The cast to u32 cannot overflow due to the length check above.
        dst.put_u32(bytes_len as u32);
        dst.put(bytes);

        Ok(())
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let mut child = Command::new(cli.program)
        .args(cli.args)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let stderr = ReaderStream::with_capacity(child.stderr.take().unwrap(), MAX).map_ok(Frame::Err);
    let stdout = ReaderStream::with_capacity(child.stdout.take().unwrap(), MAX).map_ok(Frame::Out);

    FramedWrite::new(io::stdout(), EoEncoder)
        .send_all(&mut stderr.merge(stdout))
        .await?;

    let status = child.wait().await?;

    if let Some(code) = status.code() {
        process::exit(code);
    }

    Ok(())
}
